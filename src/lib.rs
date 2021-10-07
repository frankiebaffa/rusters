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
        activeflag::ActiveFlag,
        dbmodel::DbModel,
        dbctx::DbCtx,
        primarykey::PrimaryKey,
        foreignkey::ForeignKey,
        foreignkey::ForeignKeyModel,
        uniquename::UniqueName,
        uniquename::UniqueNameModel,
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
    NoSessionError,
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
            RustersError::NoSessionError => {
                f.write_str("The session is expired or does not exist")
            }
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
    fn get_session_hash<'a>() -> Result<String, RustersError> {
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
}
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Sessions", alias="session"))]
pub struct Session {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Hash", insertable))]
    hash: String,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
    #[dbcolumn(column(name="Expired_DT", insertable))]
    expired_dt: DateTime<Utc>,
}
impl Session {
    pub fn create_new(db: &mut impl DbCtx) -> Result<Session, RustersError> {
        let hash = Hasher::get_session_hash()?;
        let session = match Session::insert_new(db, hash, Utc::now(), Utc::now() + Duration::hours(1)) {
            Ok(s) => s,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        return Ok(session);
    }
    pub fn delete_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str, value: &'a str) -> Result<(), RustersError> {
        let sql = format!("
            update {}.{}
            set {}.{} = 0
            where {}.{} = :fk
            and {}.{} = {}",
            SessionCookie::DB, SessionCookie::ALIAS,
            SessionCookie::ALIAS, SessionCookie::ACTIVE,
            SessionCookie::ALIAS, SessionCookie::FOREIGN_KEY,
            SessionCookie::ALIAS, "Name", name,
        );
    }
    pub fn create_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str, value: &'a str) -> Result<SessionCookie, RustersError> {
        let cookie = match SessionCookie::insert_new(db, self.get_id(), name.to_string(), value.to_string(), Utc::now()) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        return Ok(cookie);
    }
    pub fn get_active<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<Session, RustersError> {
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
                Err(_) => return Err(RustersError::NoSessionError),
            };
        }
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        session.update_expired(db, exp)?;
        return Ok(session);
    }
    const LOGIN_COOKIE: &'static str = "LOGIN";
    pub fn read_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str) -> Result<Option<String>, RustersError> {
        let cookies: Vec<SessionCookie> = match SessionCookie::get_all_by_fk(db, self) {
            Ok(c) => c,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let cookie_opt: Option<SessionCookie> = cookies.into_iter().filter(|cookie| {
            cookie.get_active().eq(&true) && cookie.get_name().eq(name)
        }).nth(0);
        if cookie_opt.is_some() {
            let cookie = cookie_opt.unwrap();
            return Ok(Some(cookie.get_value()));
        } else {
            return Ok(None);
        }
    }
    pub fn is_logged_in<'s>(&self, db: &mut impl DbCtx) -> Result<Option<User>, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let login_cookie_opt = self.read_cookie(db, Self::LOGIN_COOKIE)?;
        if login_cookie_opt.is_none() {
            return Ok(None);
        }
        let login_cookie = login_cookie_opt.unwrap();
        let user = match User::get_by_name(db, &login_cookie) {
            Ok(user) => user,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        return Ok(Some(user));
    }
    pub fn login<'a>(&self, db: &mut impl DbCtx, username: &'a str, password: &'a str) -> Result<User, RustersError> {
        self.update_expired(db, Utc::now())?;
        let user = match User::get_by_name(db, username) {
            Ok(user) => user,
            Err(e) => return Err(RustersError::SQLError(e)),
        };
        let stored_hash = user.get_password_hash();
        let verified = Hasher::verify(password, &stored_hash)?;
        if !verified {
            return Err(RustersError::InvalidCredentialsError);
        }
        self.create_cookie(db, Session::LOGIN_COOKIE, &username)?;
        return Ok(user);
    }
    pub fn log_out<'a>(&self, db: &mut impl DbCtx) -> Result<bool, RustersError> {
        self.update_expired(db, Utc::now())?;
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
    #[dbcolumn(column(name="Name", insertable, unique_name))]
    name: String,
    #[dbcolumn(column(name="Value", insertable))]
    value: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
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

