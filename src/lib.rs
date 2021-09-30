use std::io::Write;
use base64::{write::EncoderStringWriter, URL_SAFE};
use sha3::{Digest, Sha3_512};
pub mod context;
#[macro_use]
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
#[derive(Clone, PartialEq)]
pub enum Clearance {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
impl Clearance {
    pub fn get_all() -> Vec<Clearance> {
        return vec![
            Clearance::Pawn,
            Clearance::Knight,
            Clearance::Bishop,
            Clearance::Rook,
            Clearance::Queen,
            Clearance::King,
        ];
    }
    pub fn get_rank(&self) -> i64 {
        match self {
            Clearance::Pawn => 5,
            Clearance::Knight => 4,
            Clearance::Bishop => 3,
            Clearance::Rook => 2,
            Clearance::Queen => 1,
            Clearance::King => 0,
        }
    }
    pub fn get_name<'a>(&self) -> &'a str {
        match self {
            Clearance::Pawn => "pawn",
            Clearance::Knight => "knight",
            Clearance::Bishop => "bishop",
            Clearance::Rook => "rook",
            Clearance::Queen => "queen",
            Clearance::King => "king",
        }
    }
    pub fn from_rank(rank: i64) -> Option<Clearance> {
        match rank {
            5 => Some(Clearance::Pawn),
            4 => Some(Clearance::Knight),
            3 => Some(Clearance::Bishop),
            2 => Some(Clearance::Rook),
            1 => Some(Clearance::Queen),
            0 => Some(Clearance::King),
            _ => None,
        }
    }
}
pub struct User {
    rowid: i64,
    pk: i64,
    username: String,
    password_hash: String,
    clearance_pk: i64,
    created_dt: String,
    clearance: Clearance,
}
impl User {
    pub fn get_clearance(&self) -> Clearance {
        return self.clearance.clone();
    }
    pub fn get_username(&self) -> String {
        return self.username.clone();
    }
    pub fn get_password_hash(&self) -> String {
        return self.password_hash.clone();
    }
    pub fn match_clearance_by_name(&self, name: String) -> bool {
        return self.clearance.get_name().eq(&name);
    }
    pub fn match_clearance_by_rank(&self, lvl: i64) -> bool {
        return self.clearance.get_rank().eq(&lvl);
    }
    fn get_by_row(row: &Row) -> Result<User, rusqlite::Error> {
        let rowid_index = match row.column_index("rowid") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let rowid = match row.get(rowid_index) {
            Ok(rowid) => rowid,
            Err(e) => return Err(e),
        };
        let pk_index = match row.column_index("pk") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let pk = match row.get(pk_index) {
            Ok(pk) => pk,
            Err(e) => return Err(e),
        };
        let username_index = match row.column_index("username") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let username = match row.get(username_index) {
            Ok(username) => username,
            Err(e) => return Err(e),
        };
        let password_hash_index = match row.column_index("password_hash") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let password_hash = match row.get(password_hash_index) {
            Ok(password_hash) => password_hash,
            Err(e) => return Err(e),
        };
        let clearance_pk_index = match row.column_index("clearance_pk") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let clearance_pk = match row.get(clearance_pk_index) {
            Ok(clearance_pk) => clearance_pk,
            Err(e) => return Err(e),
        };
        let clearance = match Clearance::from_rank(clearance_pk) {
            Some(clearance) => clearance,
            None => return Err(rusqlite::Error::InvalidColumnName("Not a real invalid column name. Failed to retrieve rank by pk".to_string())),
        };
        let created_dt_index = match row.column_index("created_dt") {
            Ok(index) => index,
            Err(e) => return Err(e),
        };
        let created_dt = match row.get(created_dt_index) {
            Ok(created_dt) => created_dt,
            Err(e) => return Err(e),
        };
        return Ok(User { rowid, pk, username, password_hash, clearance_pk, created_dt, clearance });
    }
    pub fn match_creds(&self, db: &Connection, username: String, password: String) -> Option<User> {
        let base = match User::hash_pw(password) {
            Some(pw) => pw,
            None => {
                println!("Failed to hash password");
                return None;
            },
        };
        return User::get_by_creds(db, username, base);
    }
    //fn get_by_pk(db: &Connection, pk: i64) -> Option<User> {
    //    let mut sql = match db.prepare("
    //        select rowid
    //        ,   pk
    //        ,   username
    //        ,   password_hash
    //        ,   clearance_pk
    //        ,   created_dt
    //        from users
    //        where pk = ?1
    //    ") {
    //        Ok(sql) => sql,
    //        Err(_) => {
    //            println!("Failed to retrieve user from credentials");
    //            return None;
    //        },
    //    };
    //    match sql.bind(1, pk) {
    //        Ok(_) => {},
    //        Err(_) => {
    //            println!("Failed to bind pk");
    //            return None;
    //        },
    //    };
    //    return match sql.next() {
    //        Ok(_) => User::from_db(sql),
    //        Err(_) => {
    //            println!("Failed to read user from credentials");
    //            return None;
    //        },
    //    };
    //}
    pub fn get_by_creds(db: &Connection, username: String, pw_hash: String) -> Option<User> {
        const select: &'static str = include_str!("../sql/scripts/users/get_by_creds.sql");
        let mut sql = match db.prepare(select) {
            Ok(sql) => sql,
            Err(e) => {
                println!("{}", e);
                return None;
            },
        };
        return match sql.query_row(&[(":username", &username), (":password_hash", &pw_hash)], |row| {
            User::get_by_row(row)
        }) {
            Ok(user) => Some(user),
            Err(_) => {
                println!("Failed to retrieve user");
                None
            },
        };
    }
    pub fn get_by_rowid(db: &Connection, rowid: i64) -> Option<User> {
        const select: &'static str = include_str!("../sql/scripts/users/get_by_row_id.sql");
        let mut sql = match db.prepare(select) {
            Ok(sql) => sql,
            Err(_) => {
                println!("Failed to prepare statement");
                return None;
            },
        };
        //let u = sql.query_row(rusqlite::named_params!{ ":rowid": rowid, }, |row| {

        //});
        panic!("Not implemented");
    }
    pub fn hash_pw(password: String) -> Option<String> {
        return Utils::hash_string(password);
    }
    pub fn create_user(db: &Connection, username: String, lvl: i64, password: String) -> Option<User> {
        panic!("Not implemented");
        //let base = match User::hash_pw(password) {
        //    Some(pw) => pw,
        //    None => {
        //        println!("Failed to hash password");
        //        return None;
        //    },
        //};
        //let clearance = match Clearance::from_rank(lvl) {
        //    Some(c) => c,
        //    None => {
        //        println!("Could not get clearance by level {}", lvl);
        //        return None;
        //    },
        //};
        //let mut sql = match db.prepare("
        //    insert into users
        //        (
        //            username,
        //            password_hash,
        //            clearance
        //        )
        //    values
        //        (
        //            ?1,
        //            ?2,
        //            ?3
        //        );
        //    select last_insert_rowid();
        //") {
        //    Ok(sql) => sql,
        //    Err(_) => {
        //        println!("Failed to prepare statement");
        //        return None;
        //    },
        //};
        //match sql.bind(1, username.as_str()) {
        //    Ok(_) => {},
        //    Err(_) => {
        //        println!("Failed to bind column username");
        //        return None;
        //    },
        //};
        //match sql.bind(2, base.as_str()) {
        //    Ok(_) => {},
        //    Err(_) => {
        //        println!("Failed to bind column password_hash");
        //        return None;
        //    },
        //};
        //match sql.bind(3, clearance.get_rank()) {
        //    Ok(_) => {},
        //    Err(_) => {
        //        println!("Failed to bind column clearance");
        //        return None;
        //    },
        //};
        //let rowid;
        //match sql.next() {
        //    Ok(_) => {
        //        rowid = match sql.read::<i64>(0) {
        //            Ok(rowid) => rowid,
        //            Err(_) => {
        //                println!("Failed to read last inserted rowid");
        //                return None;
        //            },
        //        };
        //    },
        //    Err(_) => {
        //        println!("Failed to retrieve last inserted rowid");
        //        return None;
        //    },
        //};
        //return User::get_by_rowid(db, rowid);
    }
}

