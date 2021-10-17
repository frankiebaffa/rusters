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
    rusqlite::Error as RusqliteError,
    std::io::{
        Cursor,
        Error as IOError,
        Read,
        Write,
    },
    worm::{
        builder::{
            Query,
            WormError,
        },
        traits::{
            dbctx::DbCtx,
            primarykey::PrimaryKey,
            uniquename::UniqueNameModel,
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
    NoSessionError,
    WormError(WormError),
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
            },
            RustersError::WormError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
        }
    }
}
impl std::error::Error for RustersError {}
pub trait MatchRustersError<T, U: std::error::Error>: Sized {
    fn quick_match(self) -> Result<T, RustersError>;
}
impl<T> MatchRustersError<T, BcryptError> for Result<T, BcryptError> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::BcryptError(e)),
        };
    }
}
impl<T> MatchRustersError<T, rusqlite::Error> for Result<T, rusqlite::Error> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::SQLError(e)),
        };
    }
}
impl<T> MatchRustersError<T, std::io::Error> for Result<T, std::io::Error> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::IOError(e)),
        };
    }
}
impl<T> MatchRustersError<T, WormError> for Result<T, WormError> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::WormError(e)),
        };
    }
}
struct Hashed {
    b64_hash: String,
    salt: String,
}
struct Hasher;
impl Hasher {
    const COST: u32 = DEFAULT_COST;
    const VERSION: Version = Version::TwoB;
    fn hash_password(pw: String) -> Result<Hashed, RustersError> {
        let hash_parts = hash_with_result(pw, Self::COST).quick_match()?;
        let salt = hash_parts.get_salt();
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(hash.as_bytes()).quick_match()?;
        let b64 = enc_write.into_inner();
        return Ok(Hashed { b64_hash: b64, salt, });
    }
    fn verify<'a>(input: &'a str, stored_b64: &'a str) -> Result<bool, RustersError> {
        let mut cur = Cursor::new(stored_b64.as_bytes());
        let mut dec_read = DecoderReader::new(&mut cur, URL_SAFE);
        let mut stored_hash = String::new();
        dec_read.read_to_string(&mut stored_hash).quick_match()?;
        return verify(input, &stored_hash).quick_match();
    }
    fn get_session_hash<'a>() -> Result<String, RustersError> {
        let now = Utc::now();
        let hash_parts = hash_with_result(format!("{}", now.format("%+")), Self::COST).quick_match()?;
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(hash.as_bytes()).quick_match()?;
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
        let user = User::insert_new(db, username.to_owned(), pw_hash, salt, clearance.pk, now).quick_match()?;
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
        let session = Session::insert_new(db, hash, Utc::now(), Utc::now() + Duration::hours(1)).quick_match()?;
        return Ok(session);
    }
    pub fn get_active<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<Session, RustersError> {
        let now: DateTime<Utc> = Utc::now();
        let session = Query::<Session>::select()
            .where_gt(Session::EXPIRED_DT, &now).and()
            .where_eq(Session::HASH, &hash)
            .execute_row(db)
            .quick_match()?;
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        session.update_expired(db, exp)?;
        return Ok(session);
    }
    const LOGIN_COOKIE: &'static str = "LOGIN";
    pub fn delete_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str) -> Result<bool, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let aug = Query::<SessionCookie>::update()
            .set(SessionCookie::ACTIVE, &0)
            .where_eq(SessionCookie::SESSION_PK, &self.pk).and()
            .where_eq(SessionCookie::NAME, &name).and()
            .where_eq(SessionCookie::ACTIVE, &1)
            .execute(db)
            .quick_match()?;
        if aug.len() == 0 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    pub fn read_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str) -> Result<Option<SessionCookie>, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let cookie_res = Query::<SessionCookie>::select()
            .where_eq(SessionCookie::SESSION_PK, &self.get_id()).and()
            .where_eq(SessionCookie::NAME, &name).and()
            .where_eq(SessionCookie::ACTIVE, &1)
            .execute_row(db);
        match cookie_res {
            Ok(c) => return Ok(Some(c)),
            Err(e) => return match e {
                WormError::NoRowsError => Ok(None),
                _ => Err(e).quick_match()?,
            },
        }
    }
    pub fn set_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str, value: &'a str) -> Result<SessionCookie, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let cookie = self.read_cookie(db, name)?;
        if cookie.is_some() {
            self.delete_cookie(db, name)?;
            let new_cookie = SessionCookie::insert_new(db, self.get_id(), name.to_string(), value.to_string(), Utc::now()).quick_match()?;
            return Ok(new_cookie);
        } else {
            return Ok(SessionCookie::insert_new(
                db, self.get_id(), name.to_string(),
                value.to_string(), Utc::now()
            ).quick_match()?);
        }
    }
    pub fn is_logged_in<'s>(&self, db: &mut impl DbCtx) -> Result<Option<User>, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let login_cookie_opt = self.read_cookie(db, Self::LOGIN_COOKIE)?;
        if login_cookie_opt.is_none() {
            return Ok(None);
        }
        let login_cookie = login_cookie_opt.unwrap();
        let user = User::get_by_name(db, &login_cookie.get_value()).quick_match()?;
        return Ok(Some(user));
    }
    pub fn login<'a>(&self, db: &mut impl DbCtx, username: &'a str, password: &'a str) -> Result<User, RustersError> {
        self.update_expired(db, Utc::now())?;
        let user = User::get_by_name(db, username).quick_match()?;
        let stored_hash = user.get_password_hash();
        let verified = Hasher::verify(password, &stored_hash)?;
        if !verified {
            return Err(RustersError::InvalidCredentialsError);
        }
        self.set_cookie(db, Session::LOGIN_COOKIE, &username)?;
        return Ok(user);
    }
    pub fn log_out<'a>(&self, db: &mut impl DbCtx) -> Result<bool, RustersError> {
        let user_opt = self.is_logged_in(db)?;
        if user_opt.is_some() {
            return Ok(self.delete_cookie(db, Self::LOGIN_COOKIE)?);
        } else {
            return Ok(false);
        }
    }
    fn update_expired(&self, db: &mut impl DbCtx, new_exp: DateTime<Utc>) -> Result<(), RustersError> {
        Query::<Session>::update()
            .set(Session::EXPIRED_DT, &new_exp)
            .where_eq(Session::PK, &self.pk)
            .execute_row(db)
            .quick_match()?;
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

