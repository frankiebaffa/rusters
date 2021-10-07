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
    lock.read_line(&mut session_hash).unwrap();
    session_hash = session_hash.trim().to_string();
    //let clearance = match Clearance::get_by_id(&mut db, 1) {
    //    Ok(c) => c,
    //    Err(e) => panic!("{}", e),
    //};
    let session = match Session::get_active(&mut db, &session_hash) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    let user_opt = match session.is_logged_in(&mut db) {
        Ok(user) => user,
        Err(e) => panic!("{}", e),
    };
    if user_opt.is_some() {
        println!("Is logged in");
    } else {
        println!("No user logged in");
    }
}
