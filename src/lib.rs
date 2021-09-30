use std::io::Write;
use base64::{write::EncoderStringWriter, URL_SAFE};
use sha3::{Digest, Sha3_512};
use worm_derive::Worm;
pub mod context;
use rusqlite::{Connection, Row};
pub struct Db;
impl Db {
    pub fn get_connection() -> Connection {
        let db = match Connection::open("./users.db") {
            Ok(db) => db,
            Err(e) => panic!("{}", e),
        };
        return db;
    }
}
struct Utils {}
impl Utils {
    fn hash_string(s: String) -> Option<String> {
        let mut hasher = Sha3_512::new();
        let bytes = s.as_bytes();
        hasher.update(bytes);
        let result = hasher.finalize();
        let slc = result.as_slice();
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        match enc_write.write_all(slc) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to base 64 encode pw hash");
                return None;
            },
        };
        return Some(enc_write.into_inner());
    }
}
pub struct Clearance {
    pk: i64,
    sequence: i64,
    name: String,
}
#[derive(Worm)]
#[dbmodel(table(db="RustersDb", name="Users", alias="user"))]
pub struct User {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Username", unique_name))]
    username: String,
    #[dbcolumn(column(name="PasswordHash"))]
    password_hash: String,
    #[dbcolumn(column(name="Clearance"))]
    clearance_pk: i64,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: String,
}
#[derive(Worm)]
#[dbmodel(table(db="RustersDb", name="Salts", alias="salt"))]
pub struct Salt {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="User_PK", foreign_key="User"))]
    user_pk: i64,
    #[dbcolumn(column(name="SaltContent"))]
    salt_content: String,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: String,
}
#[derive(Worm)]
#[dbmodel(table(db="RustersDb", name="Sessions", alias="session"))]
pub struct Session {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="User_PK", foreign_key="User"))]
    user_pk: i64,
    #[dbcolumn(column(name="Hash", unique_name))]
    hash: String,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: String,
    #[dbcolumn(column(name="Expired_DT"))]
    expired_dt: String,
}

