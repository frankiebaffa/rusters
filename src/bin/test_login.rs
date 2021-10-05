use rusters::User;
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
    let mut username = String::new();
    println!("Enter username:");
    match lock.read_line(&mut username) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    username = username.trim().to_string();
    println!("Enter password:");
    let mut password = String::new();
    match lock.read_line(&mut password) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
    password = password.trim().to_string();
    //let clearance = match Clearance::get_by_id(&mut db, 1) {
    //    Ok(c) => c,
    //    Err(e) => panic!("{}", e),
    //};
    let session_hash = match User::login(&mut db, &username, &password) {
        Ok(hash) => hash,
        Err(e) => panic!("{}", e),
    };
    println!("Logged in user\r\nusername: {}\r\nsession hash: {}", username, session_hash);
}
