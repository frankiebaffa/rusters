use {
    base64::{
        write::EncoderStringWriter,
        read::DecoderReader,
        URL_SAFE,
    },
    bcrypt::{
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
    rusqlite::{
        Error,
        named_params,
    },
    std::io::{
        Cursor,
        Read,
        Write,
    },
    worm::{
        DbContext,
        traits::{
            dbmodel::DbModel,
            dbctx::DbCtx,
            primarykey::PrimaryKey,
            uniquename::{
                UniqueNameModel,
                UniqueName,
            },
        },
    },
    worm_derive::{
        Worm,
        WormDb
    },
};
pub mod context;
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
pub struct Database {
    pub context: DbContext,
}
struct Hashed {
    b64_hash: String,
    salt: String,
}
struct Hasher;
impl Hasher {
    const COST: u32 = DEFAULT_COST;
    const VERSION: Version = Version::TwoB;
    fn hash_password(pw: String) -> Hashed {
        let hash_parts = match hash_with_result(pw, Self::COST) {
            Ok(h) => h,
            Err(e) => panic!("{}", e),
        };
        let salt = hash_parts.get_salt();
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(hash.as_bytes()) {
            Ok(b) => b,
            Err(e) => panic!("{}", e),
        }
        let b64 = enc_write.into_inner();
        return Hashed { b64_hash: b64, salt, };
    }
    fn verify<'a>(input: &'a str, stored_b64: &'a str) -> bool {
        let mut cur = Cursor::new(stored_b64.as_bytes());
        let mut dec_read = DecoderReader::new(&mut cur, URL_SAFE);
        let mut stored_hash = String::new();
        match dec_read.read_to_string(&mut stored_hash) {
            Ok(_) => {},
            Err(e) => panic!("{}", e),
        }
        return match verify(input, &stored_hash) {
            Ok(b) => b,
            Err(e) => panic!("{}", e),
        };
    }
    fn get_session_hash<'a>(username: &'a str, salt: &'a str) -> String {
        let now: DateTime<Utc> = Utc::now();
        let hash_parts = match hash_with_result(format!("|{}|{}|{}|",username, salt, now.format("%+")), Self::COST) {
            Ok(hash_parts) => hash_parts,
            Err(e) => panic!("{}", e),
        };
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(hash.as_bytes()) {
            Ok(b) => b,
            Err(e) => panic!("{}", e),
        }
        let b64 = enc_write.into_inner();
        return b64;
    }
}
#[derive(Worm)]
#[dbmodel(table(db="Database", schema="RustersDb", name="Clearances", alias="clearance"))]
pub struct Clearance {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Sequence"))]
    sequence: i64,
    #[dbcolumn(column(name="Name"))]
    name: String,
}
#[derive(Worm)]
#[dbmodel(table(db="Database", schema="RustersDb", name="Users", alias="user"))]
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
    pub fn create<'a>(db: &mut Database, username: &'a str, password: &'a str, clearance: Clearance) -> Result<Self, Error> {
        let hashed = Hasher::hash_password(password.to_owned());
        let salt = hashed.salt;
        let pw_hash = hashed.b64_hash;
        let now = Utc::now();
        let user = User::insert_new(db, username.to_owned(), pw_hash, salt, clearance.pk, now)?;
        return Ok(user);
    }
    pub fn login<'a>(db: &mut Database, username: &'a str, password: &'a str) -> String {
        let user = User::get_by_name(db, username).unwrap();
        let stored_hash = user.get_password_hash();
        let verified = Hasher::verify(password, &stored_hash);
        if !verified {
            panic!("Failed to verify credentials");
        }
        let now = Utc::now();
        let exp = now + Duration::hours(1);
        let session = match Session::insert_new(
            db, user.pk,
            Hasher::get_session_hash(&user.get_name(), &user.get_salt()),
            now, exp
        ) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };
        return session.get_hash();
    }
}
#[derive(Worm)]
#[dbmodel(table(db="Database", schema="RustersDb", name="Sessions", alias="session"))]
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
    pub fn is_logged_in<'a>(db: &mut Database, hash: &'a str) -> Option<Session> {
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
                Err(e) => panic!("{}", e),
            };
            let now: DateTime<Utc> = Utc::now();
            session = match stmt.query_row(named_params!{ ":now": now, ":hash": hash, }, |row| {
                Session::from_row(&row)
            }) {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            };
        }
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        match session.update_expired(db, exp) {
            Ok(_) => {},
            Err(e) => panic!("{}", e),
        };
        return Some(session);
    }
    pub fn log_out<'a>(db: &mut Database, hash: &'a str) -> bool {
        let session_res = Session::is_logged_in(db, hash);
        if session_res.is_none() {
            return false;
        }
        let session = session_res.unwrap();
        match session.update_expired(db, Utc::now()) {
            Ok(_) => {},
            Err(e) => panic!("{}", e),
        }
        return true;
    }
    fn update_expired(&self, db: &mut Database, new_exp: DateTime<Utc>) -> Result<(), Error> {
        let sql = format!(
            "update {}.{} set Expired_DT = :dt where PK = :pk",
            Self::DB, Self::TABLE,
        );
        let c = db.use_connection();
        let mut stmt = c.prepare(&sql)?;
        stmt.execute(named_params!{ ":dt": new_exp, ":pk": self.get_id() })?;
        Ok(())
    }
}
#[derive(Worm)]
#[dbmodel(table(db="Database",schema="RustersDb",name="SessionCookies",alias="sessioncookie"))]
pub struct SessionCookie {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Session_PK", foreign_key="Session", insertable))]
    session_pk: i64,
    #[dbcolumn(column(name="Name", insertable))]
    name: String,
    #[dbcolumn(column(name="Value", insertable))]
    value: String,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl SessionCookie {
    pub fn create_or_update<'a>(db: &mut Database, session_hash: &'a str, name: &'a str, value: &'a str) {
        let session_res = Session::is_logged_in(db, session_hash);
        if session_res.is_none() {
            panic!("Not logged in");
        }
        let session = session_res.unwrap();
        let existing_cookie = Self::read(db, &session, name);
        if existing_cookie.is_none() {
            let now = Utc::now();
            match SessionCookie::insert_new(db, session.pk, name.to_string(), value.to_string(), now) {
                Ok(_) => {},
                Err(e) => panic!("{}", e),
            };
            return;
        }
        Self::update(db, session_hash, name, value);
    }
    pub fn read_value<'a>(db: &mut Database, session_hash: &'a str, name: &'a str) -> Option<String> {
        let session_res = Session::is_logged_in(db, session_hash);
        if session_res.is_none() {
            panic!("Not logged in");
        }
        let session = session_res.unwrap();
        return Self::read(db, &session, name);
    }
    fn read<'a>(db: &mut Database, session: &Session, name: &'a str) -> Option<String> {
        let sql = format!("
            select {}.*
            from {}.{} as {}
            where {}.Session_PK = :pk
            and {}.Name = :name
            limit 1;",
            Self::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Self::ALIAS,
            Self::ALIAS,
        );
        let c = db.use_connection();
        let mut stmt = match c.prepare(&sql) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };
        let cookies: Vec<Result<SessionCookie, Error>> = match stmt.query_map(named_params!{ ":pk": session.get_id(), ":name": name, }, |row| {
            Self::from_row(&row)
        }) {
            Ok(c) => c,
            Err(e) => panic!("{}", e),
        }.collect();
        if cookies.len() == 0 {
            return None;
        }
        let cookie_res = cookies.into_iter().nth(0).unwrap();
        let cookie = cookie_res.unwrap();
        return Some(cookie.value);
    }
    fn update<'a>(db: &mut Database, session_hash: &'a str, name: &'a str, value: &'a str) {
        let sql = format!("
            update {}.{}
            set Value = :value
            from {}.{} as {}
            join {}.{} as {}
            on {}.PK = {}.Session_PK
            and {}.Hash = :hash
            and {}.Name = :name
            and {}.Expired_DT > :now;",
            Self::DB, Self::TABLE,
            Session::DB, Session::TABLE, Session::ALIAS,
            Self::DB, Self::TABLE, Self::ALIAS,
            Session::ALIAS, Self::ALIAS,
            Session::ALIAS,
            Self::ALIAS,
            Session::ALIAS,
        );
        let c = db.use_connection();
        c.execute(&sql, named_params!{ ":value": value, ":hash": session_hash, ":name": name, ":now": Utc::now(), }).unwrap();
    }
}

