use rusters::SessionCookie;
use rusters::Database;
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
    let cookie_res = SessionCookie::read_value(&mut db, &session_hash, "Test");
    if cookie_res.is_none() {
        println!("No value for cookie");
    } else {
        println!("Cookie found!\r\nvalue: {}", cookie_res.unwrap());
    }
}
