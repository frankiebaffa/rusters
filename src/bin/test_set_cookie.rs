use rusters::Database;
use rusters::SessionCookie;
use worm::DbCtx;
use std::io::BufRead;
fn main() {
    let mut db = Database::init();
    db.context.attach_dbs();
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    println!("Enter session hash:");
    let mut session_hash = String::new();
    match lock.read_line(&mut session_hash) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    session_hash = session_hash.trim().to_string();
    SessionCookie::create_or_update(&mut db, &session_hash, "Test", "Hello, World!");
    println!("Created cookie");
}
