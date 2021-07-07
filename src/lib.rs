use std::io::Write;
use base64::{write::EncoderStringWriter, URL_SAFE};
use sha3::{Digest, Sha3_512};
use sqlite::{Connection, Statement, State};
pub struct Db;
impl Db {
    pub fn get_connection() -> Connection {
        let db = match Connection::open("./users.rs") {
            Ok(db) => db,
            Err(e) => panic!("{}", e),
        };
        Db::check_db(&db);
        return db;
    }
    fn check_db(db: &Connection) {
        let init_db_sql = include_str!("../sql/up.sql");
        let stmt_res = db.prepare(init_db_sql);
        let mut stmt = match stmt_res {
            Ok(stmt) => stmt,
            Err(e) => panic!("{}", e),
        };
        match stmt.next() {
            Ok(state) => match state {
                State::Row => panic!("Db creation statement returned more results than expected"),
                State::Done => return,
            },
            Err(e) => panic!("{}", e),
        };
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
enum Clearance {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
impl Clearance {
    fn get_rank(&self) -> i64 {
        match self {
            Clearance::Pawn => 5,
            Clearance::Knight => 4,
            Clearance::Bishop => 3,
            Clearance::Rook => 2,
            Clearance::Queen => 1,
            Clearance::King => 0,
        }
    }
    fn get_name<'a>(&self) -> &'a str {
        match self {
            Clearance::Pawn => "pawn",
            Clearance::Knight => "knight",
            Clearance::Bishop => "bishop",
            Clearance::Rook => "rook",
            Clearance::Queen => "queen",
            Clearance::King => "king",
        }
    }
    fn from_rank(rank: i64) -> Option<Clearance> {
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
    pub fn get_username(&self) -> String {
        return self.username.clone();
    }
    pub fn match_clearance_by_name(&self, name: String) -> bool {
        return self.clearance.get_name().eq(&name);
    }
    pub fn match_clearance_by_rank(&self, lvl: i64) -> bool {
        return self.clearance.get_rank().eq(&lvl);
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
    fn from_db(sql: Statement) -> Option<User> {
        let rowid = match sql.read::<i64>(0) {
            Ok(rowid) => rowid,
            Err(_) => {
                println!("Failed to read rowid");
                return None;
            },
        };
        let pk = match sql.read::<i64>(1) {
            Ok(pk) => pk,
            Err(_) => {
                println!("Failed to read pk");
                return None;
            },
        };
        let username = match sql.read::<String>(2) {
            Ok(username) => username,
            Err(_) => {
                println!("Failed to read username");
                return None;
            },
        };
        let password_hash = match sql.read::<String>(3) {
            Ok(password_hash) => password_hash,
            Err(_) => {
                println!("Failed to read password_hash");
                return None;
            },
        };
        let clearance_pk = match sql.read::<i64>(4) {
            Ok(clearance_pk) => clearance_pk,
            Err(_) => {
                println!("Failed to read clearance_pk");
                return None;
            },
        };
        let created_dt = match sql.read::<String>(5) {
            Ok(created_dt) => created_dt,
            Err(_) => {
                println!("Failed to read created_dt");
                return None;
            },
        };
        let clearance = match Clearance::from_rank(clearance_pk) {
            Some(clearance) => clearance,
            None => {
                println!("Failed to retrieve clearance by clearance_pk");
                return None;
            },
        };
        return Some(User { rowid, pk, username, password_hash, clearance_pk, created_dt, clearance, });
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
        let mut sql = match db.prepare("
            select rowid
            ,   pk
            ,   username
            ,   password_hash
            ,   clearance_pk
            ,   created_dt
            from users
            where username = ?1
            and password_hash = ?2
        ") {
            Ok(sql) => sql,
            Err(_) => {
                println!("Failed to retrieve user from credentials");
                return None;
            },
        };
        match sql.bind(1, username.as_str()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind username");
                return None;
            },
        };
        match sql.bind(2, pw_hash.as_str()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind password_hash");
                return None;
            },
        };
        return match sql.next() {
            Ok(_) => User::from_db(sql),
            Err(_) => {
                println!("Failed to read user from credentials");
                return None;
            },
        };
    }
    pub fn get_by_rowid(db: &Connection, rowid: i64) -> Option<User> {
        let mut sql = match db.prepare("
            select rowid
            ,   pk
            ,   username
            ,   password_hash
            ,   clearance_pk
            ,   created_dt
            from users
            where rowid = ?1
        ") {
            Ok(sql) => sql,
            Err(_) => {
                println!("Failed to prepare statement");
                return None;
            },
        };
        match sql.bind(1, rowid) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind column rowid");
                return None;
            },
        };
        return match sql.next() {
            Ok(_) => User::from_db(sql),
            Err(_) => {
                println!("Failed to read user by rowid {}", rowid);
                return None;
            },
        };
    }
    fn hash_pw(password: String) -> Option<String> {
        return Utils::hash_string(password);
    }
    pub fn create_user(db: &Connection, username: String, lvl: i64, password: String) -> Option<User> {
        let base = match User::hash_pw(password) {
            Some(pw) => pw,
            None => {
                println!("Failed to hash password");
                return None;
            },
        };
        let clearance = match Clearance::from_rank(lvl) {
            Some(c) => c,
            None => {
                println!("Could not get clearance by level {}", lvl);
                return None;
            },
        };
        let mut sql = match db.prepare("
            insert into users
                (
                    username,
                    password_hash,
                    clearance_pk
                )
            values
                (
                    ?1,
                    ?2,
                    ?3
                );
            select last_insert_rowid();
        ") {
            Ok(sql) => sql,
            Err(_) => {
                println!("Failed to prepare statement");
                return None;
            },
        };
        match sql.bind(1, username.as_str()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind column username");
                return None;
            },
        };
        match sql.bind(2, base.as_str()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind column password_hash");
                return None;
            },
        };
        match sql.bind(3, clearance.get_rank()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to bind column clearance");
                return None;
            },
        };
        let rowid;
        match sql.next() {
            Ok(_) => {
                rowid = match sql.read::<i64>(0) {
                    Ok(rowid) => rowid,
                    Err(_) => {
                        println!("Failed to read last inserted rowid");
                        return None;
                    },
                };
            },
            Err(_) => {
                println!("Failed to retrieve last inserted rowid");
                return None;
            },
        };
        return User::get_by_rowid(db, rowid);
    }
}

