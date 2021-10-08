use rusters::Session;
use worm::{DbCtx, DbContext};
use std::io::BufRead;
use worm_derive::WormDb;
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
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
    let session = match Session::get_active(&mut db, &session_hash) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    match session.delete_cookie(&mut db, "Test") {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };
    println!("Cookie deleted!");
}
