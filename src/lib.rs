use {
    base64::{
        write::EncoderStringWriter,
        URL_SAFE
    },
    chrono::{
        DateTime,
        Local,
        Utc
    },
    rusqlite::{
        Error,
        named_params,
    },
    sha3::{
        Digest,
        Sha3_512
    },
    std::io::{
        Write,
        Result as IOResult
    },
    worm::{
        DbContext,
        traits::{
            dbmodel::DbModel,
            dbctx::DbCtx,
            foreignkey::ForeignKeyModel,
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
struct HashUtils {}
impl HashUtils {
    fn hash_string(s: String) -> IOResult<String> {
        let mut hasher = Sha3_512::new();
        let bytes = s.as_bytes();
        hasher.update(bytes);
        let result = hasher.finalize();
        let slc = result.as_slice();
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(slc)?;
        return Ok(enc_write.into_inner());
    }
    fn get_salt_str<'a>(username: &'a str) -> IOResult<String> {
        let utc_now = Utc::now();
        let now: DateTime<Local> = utc_now.into();
        let now_str = now.format("%+").to_string();
        let to_salt = format!("{}|{}", username, now_str);
        return Self::hash_string(to_salt);
    }
    fn hash_password<'a>(pw: &'a str, salt: String) -> IOResult<String> {
        let formatted = format!("{}{}", pw, salt);
        return Self::hash_string(formatted);
    }
}
#[derive(Worm)]
#[dbmodel(table(db="Database", schema="RustersDb", name="Clearance", alias="clearance"))]
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
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Clearance", insertable))]
    clearance_pk: i64,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: DateTime<Local>,
}
impl User {
    pub fn create<'a>(db: &mut Database, username: &'a str, password: &'a str, clearance: Clearance) -> Result<Self, Error> {
        let salt = match HashUtils::get_salt_str(username) {
            Ok(salt) => salt,
            Err(e) => panic!("{}", e),
        };
        let pw_hash = match HashUtils::hash_password(password, salt.clone()) {
            Ok(pw) => pw,
            Err(e) => panic!("{}", e),
        };
        let user = User::insert_new(db, username.to_owned(), pw_hash, clearance.get_id())?;
        Salt::insert_new(db, user.get_id(), salt)?;
        return Ok(user);
    }
    pub fn login<'a>(db: &mut Database, username: &'a str, password: &'a str) -> Option<String> {
        let user = User::get_by_name(db, username).unwrap();
        let salts = Salt::get_all_by_fk(db, &user).unwrap();
        let salt = if salts.len() > 0 {
            salts.get(0).unwrap()
        } else {
            panic!("Failed to retrieve user salt");
        };
        let act_pw = user.password_hash.clone();
        let pw = match HashUtils::hash_password(password, salt.get_salt_content()) {
            Ok(pw) => pw,
            Err(e) => panic!("{}", e),
        };
        if pw.eq(&act_pw) {
            let salt = match HashUtils::get_salt_str(&user.get_name()) {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            };
            let s = match Session::insert_new(db, user.get_id(), salt) {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            };
            return Some(s.get_hash());
        } else {
            return None;
        }
    }
}
#[derive(Worm)]
#[dbmodel(table(db="Database", schema="RustersDb", name="Salts", alias="salt"))]
pub struct Salt {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="User_PK", foreign_key="User", insertable))]
    user_pk: i64,
    #[dbcolumn(column(name="SaltContent", insertable))]
    salt_content: String,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: DateTime<Local>,
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
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: DateTime<Local>,
    #[dbcolumn(column(name="Expired_DT"))]
    expired_dt: DateTime<Local>,
}
impl Session {
    pub fn is_logged_in<'a>(db: &mut Database, hash: &'a str) -> Result<bool, Error> {
        let sql = format!(
            "select {}.* from {}.{} as {} where {}.Expired_DT < :now and {}.Hash = :hash limit 1;",
            Self::ALIAS, Self::DB, Self::TABLE, Self::ALIAS, Self::ALIAS, Self::ALIAS,
        );
        let session;
        {
            let c = db.use_connection();
            let mut stmt = c.prepare(&sql)?;
            let now: DateTime<Local> = Utc::now().into();
            session = stmt.query_row(named_params!{ ":now": now, ":hash": hash }, |row| {
                Session::from_row(&row)
            })?;
        }
        session.update_expired(db)?;
        return Ok(true);
    }
    fn update_expired(&self, db: &mut Database) -> Result<(), Error> {
        let sql = format!(
            "update {}.{} set Expired_DT = :dt where PK = :pk",
            Self::DB, Self::TABLE,
        );
        let c = db.use_connection();
        let mut stmt = c.prepare(&sql)?;
        let now: DateTime<Local> = Utc::now().into();
        stmt.execute(named_params!{ ":dt": now, ":pk": self.get_id() })?;
        Ok(())
    }
}

