use {
    base64::{
        write::EncoderStringWriter,
        read::DecoderReader,
        URL_SAFE,
    },
    bcrypt::{
        BcryptError,
        DEFAULT_COST,
        hash_with_result,
        verify,
        Version,
    },
    chrono::{
        DateTime,
        Duration,
        Utc,
    },
    migaton::Migrator,
    rusqlite::{
        Error as RusqliteError,
        named_params,
    },
    std::io::{
        Cursor,
        Error as IOError,
        Read,
        Write,
    },
    worm::traits::{
        dbmodel::DbModel,
        dbctx::DbCtx,
        primarykey::PrimaryKey,
        uniquename::{
            UniqueNameModel,
            UniqueName,
        },
    },
    worm_derive::Worm,
};
pub mod context;
#[derive(Debug)]
pub enum RustersError {
    BcryptError(BcryptError),
    InvalidCredentialsError,
    IOError(IOError),
    NotLoggedInError,
    SQLError(RusqliteError),
}
impl std::fmt::Display for RustersError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustersError::BcryptError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
            RustersError::InvalidCredentialsError => {
                f.write_str("Invalid credentials")
            },
            RustersError::IOError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
            RustersError::NotLoggedInError => {
                f.write_str("Not logged in")
            },
            RustersError::SQLError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
        }
    }
}
impl std::error::Error for RustersError {}
struct Hashed {
    b64_hash: String,
    salt: String,
}
struct Hasher;
impl Hasher {
    const COST: u32 = DEFAULT_COST;
    const VERSION: Version = Version::TwoB;
    fn hash_password(pw: String) -> Result<Hashed, RustersError> {
        let hash_parts = match hash_with_result(pw, Self::COST) {
            Ok(h) => h,
            Err(e) => return Err(RustersError::BcryptError(e)),
        };
        let salt = hash_parts.get_salt();
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(hash.as_bytes()) {
            Ok(b) => b,
            Err(e) => return Err(RustersError::IOError(e)),
        }
        let b64 = enc_write.into_inner();
        return Ok(Hashed { b64_hash: b64, salt, });
    }
    fn verify<'a>(input: &'a str, stored_b64: &'a str) -> Result<bool, RustersError> {
        let mut cur = Cursor::new(stored_b64.as_bytes());
        let mut dec_read = DecoderReader::new(&mut cur, URL_SAFE);
        let mut stored_hash = String::new();
        match dec_read.read_to_string(&mut stored_hash) {
            Ok(_) => {},
            Err(e) => return Err(RustersError::IOError(e)),
        }
        return match verify(input, &stored_hash) {
            Ok(b) => Ok(b),
            Err(e) => return Err(RustersError::BcryptError(e)),
        };
    }
    fn get_session_hash<'a>(username: &'a str, salt: &'a str) -> Result<String, RustersError> {
        let now = Utc::now();
        let hash_parts = match hash_with_result(format!("|{}|{}|{}|",username, salt, now.format("%+")), Self::COST) {
            Ok(hash_parts) => hash_parts,
            Err(e) => return Err(RustersError::BcryptError(e)),
        };
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(hash.as_bytes()) {
            Ok(b) => b,
            Err(e) => return Err(RustersError::IOError(e)),
        }
        let b64 = enc_write.into_inner();
        return Ok(b64);
    }
    fn get_open_session_hash<'a>() -> Result<String, RustersError> {
        let now = Utc::now();
        let hash_parts = match hash_with_result(format!("{}", now.format("%+")), Self::COST) {
            Ok(hash_parts) => hash_parts,
            Err(e) => return Err(RustersError::BcryptError(e)),
        };
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(hash.as_bytes()) {
            Ok(b) => b,
            Err(e) => return Err(RustersError::IOError(e)),
        }
        let b64 = enc_write.into_inner();
        return Ok(b64);
    }
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Clearances", alias="clearance"))]
pub struct Clearance {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Sequence"))]
    sequence: i64,
    #[dbcolumn(column(name="Name"))]
    name: String,
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Users", alias="user"))]
pub struct User {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Username", insertable, unique_name))]
    username: String,
    #[dbcolumn(column(name="PasswordHash", insertable))]
    password_hash: String,
    #[dbcolumn(column(name="Salt", insertable))]
    salt: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Clearance_PK", insertable))]
    clearance_pk: i64,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl User {
    pub fn create<'a>(db: &mut impl DbCtx, username: &'a str, password: &'a str, clearance: Clearance) -> Result<Self, RustersError> {
        let hashed = Hasher::hash_password(password.to_owned())?;
        let salt = hashed.salt;
        let pw_hash = hashed.b64_hash;
        let now = Utc::now();
        let user = match User::insert_new(db, username.to_owned(), pw_hash, salt, clearance.pk, now) {
            Ok(user) => user,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        return Ok(user);
    }
    pub fn login<'a>(db: &mut impl DbCtx, username: &'a str, password: &'a str) -> Result<String, RustersError> {
        let user = match User::get_by_name(db, username) {
            Ok(user) => user,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let stored_hash = user.get_password_hash();
        let verified = Hasher::verify(password, &stored_hash)?;
        if !verified {
            return Err(RustersError::InvalidCredentialsError);
        }
        let now = Utc::now();
        let exp = now + Duration::hours(1);
        let session = match Session::insert_new(
            db, user.pk,
            Hasher::get_session_hash(&user.get_name(), &user.get_salt())?,
            now, exp
        ) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        return Ok(session.get_hash());
    }
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="OpenSessions", alias="session"))]
pub struct OpenSession {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Hash", insertable))]
    hash: String,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
    #[dbcolumn(column(name="Expired_DT", insertable))]
    expired_dt: DateTime<Utc>,
}
impl OpenSession {
    pub fn create_new<'a>(db: &mut impl DbCtx) -> Result<OpenSession, RustersError> {
        let hash = Hasher::get_open_session_hash()?;
        return match OpenSession::insert_new(db, hash, Utc::now(), Utc::now() + Duration::hours(1)) {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::SQLError(e)),
        };
    }
    pub fn get_active<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<Option<OpenSession>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            where {}.Expired_DT > :now
            and {}.Hash = :hash
            limit 1;",
            Self::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
        );
        let session;
        {
            let c = db.use_connection();
            let mut stmt = match c.prepare(&sql) {
                Ok(stmt) => stmt,
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            let now: DateTime<Utc> = Utc::now();
            session = match stmt.query_row(named_params!{ ":now": now, ":hash": hash, }, |row| {
                OpenSession::from_row(&row)
            }) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            };
        }
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        session.update_expired(db, exp)?;
        return Ok(Some(session));
    }
    fn update_expired(&self, db: &mut impl DbCtx, new_exp: DateTime<Utc>) -> Result<(), RustersError> {
        let sql = format!(
            "update {}.{} set Expired_DT = :dt where PK = :pk",
            Self::DB, Self::TABLE,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(stmt) => stmt,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        match stmt.execute(named_params!{ ":dt": new_exp, ":pk": self.get_id() }) {
            Ok(_) => {},
            Err(e) => return Err(RustersError::SQLError(e)),
        }
        Ok(())
    }
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="OpenSessionCookies", alias="opensessioncookie"))]
pub struct OpenSessionCookie {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="OpenSession_PK", foreign_key="OpenSession", insertable))]
    opensession_pk: i64,
    #[dbcolumn(column(name="Name", insertable))]
    name: String,
    #[dbcolumn(column(name="Value", insertable))]
    value: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl OpenSessionCookie {
    pub fn create_or_update<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str, value: &'a str) -> Result<(), RustersError> {
        let session_res = OpenSession::get_active(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        let existing_cookie = Self::read(db, &session, name)?;
        if existing_cookie.is_none() {
            let now = Utc::now();
            match OpenSessionCookie::insert_new(db, session.pk, name.to_string(), value.to_string(), now) {
                Ok(_) => {},
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            return Ok(());
        }
        Self::update(db, session_hash, name, value);
        Ok(())
    }
    pub fn read_value<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) -> Result<Option<String>, RustersError> {
        let session_res = OpenSession::get_active(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        return Ok(Self::read(db, &session, name)?);
    }
    pub fn get_alerts<'a>(db: &mut impl DbCtx, session_hash: &'a str) -> Result<Vec<Self>, RustersError> {
        let session_res = OpenSession::get_active(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        return Ok(Self::alerts(db, &session)?);
    }
    pub fn delete_cookie<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) -> Result<(), RustersError> {
        let session_res = OpenSession::get_active(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        Self::delete(db, session_hash, name);
        Ok(())
    }
    fn read<'a>(db: &mut impl DbCtx, session: &OpenSession, name: &'a str) -> Result<Option<String>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            where {}.OpenSession_PK = :pk
            and {}.Name = :name
            and {}.Active = 1
            limit 1;",
            Self::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let cookies: Vec<Result<OpenSessionCookie, RusqliteError>> = match stmt.query_map(named_params!{ ":pk": session.get_id(), ":name": name, }, |row| {
            Self::from_row(&row)
        }) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        }.collect();
        if cookies.len() == 0 {
            return Ok(None);
        }
        let cookie_res = cookies.into_iter().nth(0).unwrap();
        let cookie = cookie_res.unwrap();
        return Ok(Some(cookie.value));
    }
    fn alerts<'a>(db: &mut impl DbCtx, session: &OpenSession) -> Result<Vec<Self>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            join {}.{} as {}
            on {}.{} = {}.OpenSession_PK
            and {}.{} = :session_pk
            and {}.Active = 1
            and {}.Name in ('success', 'error', 'alert');",
            Self::ALIAS,
            OpenSession::DB, OpenSession::TABLE, OpenSession::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            OpenSession::ALIAS, OpenSession::PRIMARY_KEY, Self::ALIAS,
            OpenSession::ALIAS, OpenSession::PRIMARY_KEY,
            Self::ALIAS,
            Self::ALIAS,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let cookies: Vec<Result<OpenSessionCookie, RusqliteError>> = match stmt.query_map(named_params!{ ":session_pk": session.get_id(), }, |row| {
            Self::from_row(&row)
        }) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        }.collect();
        if cookies.len() == 0 {
            return Ok(Vec::new());
        }
        let mut alerts = Vec::new();
        for cookie_res in cookies {
            let c = match cookie_res {
                Ok(c) => c,
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            alerts.push(c);
        }
        return Ok(alerts);
    }
    fn update<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str, value: &'a str) {
        let sql = format!("
            update {}.{}
            set Value = :value
            from {}.{} as {}
            join {}.{} as {}
            on {}.PK = {}.OpenSession_PK
            and {}.Hash = :hash
            and {}.Name = :name
            and {}.Active = 1
            and {}.Expired_DT > :now;",
            Self::DB, Self::TABLE,
            OpenSession::DB, OpenSession::TABLE, OpenSession::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            OpenSession::ALIAS, Self::ALIAS,
            OpenSession::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            OpenSession::ALIAS,
        );
        let c = db.use_connection();
        c.execute(&sql, named_params!{ ":value": value, ":hash": session_hash, ":name": name, ":now": Utc::now(), }).unwrap();
    }
    fn delete<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) {
        let sql = format!("
            update {}.{}
            set Active = 0
            from {}.{} as {}
            join {}.{} as {}
            on {}.PK = {}.OpenSession_PK
            and {}.Hash = :hash
            and {}.Name = :name
            and {}.Active = 1
            and {}.Expired_DT > :now;",
            Self::DB, Self::TABLE,
            OpenSession::DB, OpenSession::TABLE, OpenSession::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            OpenSession::ALIAS, Self::ALIAS,
            OpenSession::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            OpenSession::ALIAS,
        );
        let c = db.use_connection();
        c.execute(&sql, named_params!{ ":hash": session_hash, ":name": name, ":now": Utc::now(), }).unwrap();
    }
    pub fn delete_self<'a>(&self, db: &mut impl DbCtx) -> Result<(), RustersError> {
        let sql = format!("
            update {}.{}
            set Active = 0
            where {}.{} = :id",
            Self::DB, Self::TABLE,
            Self::TABLE, Self::PRIMARY_KEY
        );
        let c = db.use_connection();
        match c.execute(&sql, named_params!{ ":id": self.get_id(), }) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(RustersError::SQLError(e)),
        };
    }
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Sessions", alias="session"))]
pub struct Session {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="User_PK", foreign_key="User", insertable))]
    user_pk: i64,
    #[dbcolumn(column(name="Hash", insertable))]
    hash: String,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
    #[dbcolumn(column(name="Expired_DT", insertable))]
    expired_dt: DateTime<Utc>,
}
impl Session {
    pub fn is_logged_in<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<Option<Session>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            where {}.Expired_DT > :now
            and {}.Hash = :hash
            limit 1;",
            Self::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
        );
        let session;
        {
            let c = db.use_connection();
            let mut stmt = match c.prepare(&sql) {
                Ok(stmt) => stmt,
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            let now: DateTime<Utc> = Utc::now();
            session = match stmt.query_row(named_params!{ ":now": now, ":hash": hash, }, |row| {
                Session::from_row(&row)
            }) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            };
        }
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        session.update_expired(db, exp)?;
        return Ok(Some(session));
    }
    pub fn log_out<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<bool, RustersError> {
        let session_res = Session::is_logged_in(db, hash)?;
        if session_res.is_none() {
            return Ok(false);
        }
        let session = session_res.unwrap();
        session.update_expired(db, Utc::now())?;
        return Ok(true);
    }
    fn update_expired(&self, db: &mut impl DbCtx, new_exp: DateTime<Utc>) -> Result<(), RustersError> {
        let sql = format!(
            "update {}.{} set Expired_DT = :dt where PK = :pk",
            Self::DB, Self::TABLE,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(stmt) => stmt,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        match stmt.execute(named_params!{ ":dt": new_exp, ":pk": self.get_id() }) {
            Ok(_) => {},
            Err(e) => return Err(RustersError::SQLError(e)),
        }
        Ok(())
    }
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb",name="SessionCookies",alias="sessioncookie"))]
pub struct SessionCookie {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Session_PK", foreign_key="Session", insertable))]
    session_pk: i64,
    #[dbcolumn(column(name="Name", insertable))]
    name: String,
    #[dbcolumn(column(name="Value", insertable))]
    value: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl SessionCookie {
    pub fn create_or_update<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str, value: &'a str) -> Result<(), RustersError> {
        let session_res = Session::is_logged_in(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        let existing_cookie = Self::read(db, &session, name)?;
        if existing_cookie.is_none() {
            let now = Utc::now();
            match SessionCookie::insert_new(db, session.pk, name.to_string(), value.to_string(), now) {
                Ok(_) => {},
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            return Ok(());
        }
        Self::update(db, session_hash, name, value);
        Ok(())
    }
    pub fn read_value<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) -> Result<Option<String>, RustersError> {
        let session_res = Session::is_logged_in(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        return Ok(Self::read(db, &session, name)?);
    }
    pub fn get_alerts<'a>(db: &mut impl DbCtx, session_hash: &'a str) -> Result<Vec<Self>, RustersError> {
        let session_res = Session::is_logged_in(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        let session = session_res.unwrap();
        return Ok(Self::alerts(db, &session)?);
    }
    pub fn delete_cookie<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) -> Result<(), RustersError> {
        let session_res = Session::is_logged_in(db, session_hash)?;
        if session_res.is_none() {
            return Err(RustersError::NotLoggedInError);
        }
        Self::delete(db, session_hash, name);
        Ok(())
    }
    fn read<'a>(db: &mut impl DbCtx, session: &Session, name: &'a str) -> Result<Option<String>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            where {}.Session_PK = :pk
            and {}.Name = :name
            and {}.Active = 1
            limit 1;",
            Self::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let cookies: Vec<Result<SessionCookie, RusqliteError>> = match stmt.query_map(named_params!{ ":pk": session.get_id(), ":name": name, }, |row| {
            Self::from_row(&row)
        }) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        }.collect();
        if cookies.len() == 0 {
            return Ok(None);
        }
        let cookie_res = cookies.into_iter().nth(0).unwrap();
        let cookie = cookie_res.unwrap();
        return Ok(Some(cookie.value));
    }
    fn alerts<'a>(db: &mut impl DbCtx, session: &Session) -> Result<Vec<Self>, RustersError> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            join {}.{} as {}
            on {}.{} = {}.Session_PK
            and {}.{} = :session_pk
            and {}.Active = 1
            and {}.Name in ('success', 'error', 'alert');",
            Self::ALIAS,
            Session::DB, Session::TABLE, Session::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Session::ALIAS, Session::PRIMARY_KEY, Self::ALIAS,
            Session::ALIAS, Session::PRIMARY_KEY,
            Self::ALIAS,
            Self::ALIAS,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let cookies: Vec<Result<SessionCookie, RusqliteError>> = match stmt.query_map(named_params!{ ":session_pk": session.get_id(), }, |row| {
            Self::from_row(&row)
        }) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        }.collect();
        if cookies.len() == 0 {
            return Ok(Vec::new());
        }
        let mut alerts = Vec::new();
        for cookie_res in cookies {
            let c = match cookie_res {
                Ok(c) => c,
                Err(e) => return Err(RustersError::SQLError(e)),
            };
            alerts.push(c);
        }
        return Ok(alerts);
    }
    fn update<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str, value: &'a str) {
        let sql = format!("
            update {}.{}
            set Value = :value
            from {}.{} as {}
            join {}.{} as {}
            on {}.PK = {}.Session_PK
            and {}.Hash = :hash
            and {}.Name = :name
            and {}.Active = 1
            and {}.Expired_DT > :now;",
            Self::DB, Self::TABLE,
            Session::DB, Session::TABLE, Session::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Session::ALIAS, Self::ALIAS,
            Session::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            Session::ALIAS,
        );
        let c = db.use_connection();
        c.execute(&sql, named_params!{ ":value": value, ":hash": session_hash, ":name": name, ":now": Utc::now(), }).unwrap();
    }
    fn delete<'a>(db: &mut impl DbCtx, session_hash: &'a str, name: &'a str) {
        let sql = format!("
            update {}.{}
            set Active = 0
            from {}.{} as {}
            join {}.{} as {}
            on {}.PK = {}.Session_PK
            and {}.Hash = :hash
            and {}.Name = :name
            and {}.Active = 1
            and {}.Expired_DT > :now;",
            Self::DB, Self::TABLE,
            Session::DB, Session::TABLE, Session::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Session::ALIAS, Self::ALIAS,
            Session::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
            Session::ALIAS,
        );
        let c = db.use_connection();
        c.execute(&sql, named_params!{ ":hash": session_hash, ":name": name, ":now": Utc::now(), }).unwrap();
    }
    pub fn delete_self<'a>(&self, db: &mut impl DbCtx) -> Result<(), RustersError> {
        let sql = format!("
            update {}.{}
            set Active = 0
            where {}.{} = :id",
            Self::DB, Self::TABLE,
            Self::TABLE, Self::PRIMARY_KEY
        );
        let c = db.use_connection();
        match c.execute(&sql, named_params!{ ":id": self.get_id(), }) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(RustersError::SQLError(e)),
        };
    }
}
pub struct Migrations;
impl Migrations {
    const MIGRATIONS_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/sql/migrations");
    pub fn migrate_up(mem_db: &mut impl DbCtx, db: &mut impl DbCtx) -> usize {
        let mut mem_c = mem_db.use_connection();
        let mut c = db.use_connection();
        let skips = match Migrator::do_up(&mut mem_c, &mut c, Self::MIGRATIONS_PATH) {
            Ok(res) => res,
            Err(e) => panic!("{}", e),
        };
        return skips;
    }
    pub fn migrate_down(mem_db: &mut impl DbCtx, db: &mut impl DbCtx) -> usize {
        let mut mem_c = mem_db.use_connection();
        let mut c = db.use_connection();
        let skips = match Migrator::do_down(&mut mem_c, &mut c, Self::MIGRATIONS_PATH) {
            Ok(res) => res,
            Err(e) => panic!("{}", e),
        };
        return skips;
    }
}

