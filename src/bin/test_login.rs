use rusters::{User, Session};
use worm::{DbCtx, DbContext, traits::uniquename::UniqueName};
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
    let session = match Session::create_new(&mut db) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    //let clearance = match Clearance::get_by_id(&mut db, 1) {
    //    Ok(c) => c,
    //    Err(e) => panic!("{}", e),
    //};
    let user = match session.login(&mut db, &username, &password) {
        Ok(u) => u,
        Err(e) => panic!("{}", e),
    };
    println!("Logged in user\r\nusername: {}\r\nsession hash: {}", user.get_name(), session.get_hash());
}
