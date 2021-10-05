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
    //let clearance = match Clearance::get_by_id(&mut db, 1) {
    //    Ok(c) => c,
    //    Err(e) => panic!("{}", e),
    //};
    let logged_out = match Session::log_out(&mut db, &session_hash) {
        Ok(b) => b,
        Err(e) => panic!("{}", e),
    };
    if logged_out {
        println!("Logged out!");
    } else {
        println!("Was not logged in.");
    }
}
