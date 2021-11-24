use rusters::Session;
use worm::core::{DbCtx, DbContext};
use std::io::BufRead;
use worm::derive::WormDb;
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
fn main() {
    match dotenv::dotenv() {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    let mut session_hash = match std::env::var("SESSION_HASH") {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    session_hash = session_hash.trim().to_string();
    let mut db = Database::init();
    db.context.attach_dbs();
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    println!("Enter cookie name:");
    let mut name = String::new();
    match lock.read_line(&mut name) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    name = name.trim().to_string();
    println!("Enter cookie value:");
    let mut value = String::new();
    match lock.read_line(&mut value) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    value = value.trim().to_string();
    let session = match Session::get_active(&mut db, &session_hash) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    match session.set_cookie(&mut db, &name, &value) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    println!("Created cookie");
}
