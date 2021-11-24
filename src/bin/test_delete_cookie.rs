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
    let session = match Session::get_active(&mut db, &session_hash) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    println!("Enter cookie name:");
    let mut name = String::new();
    match lock.read_line(&mut name) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    name = name.trim().to_string();
    let deleted = match session.delete_cookie(&mut db, &name) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };
    if deleted {
        println!("Cookie deleted!");
    } else {
        println!("Failed to delete");
    }
}
