use rusters::Clearance;
use rusters::User;
use rusters::Database;
use worm::DbCtx;
use worm::traits::primarykey::PrimaryKeyModel;
use worm::traits::uniquename::UniqueName;
use std::io::BufRead;
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
    let clearance = match Clearance::get_by_id(&mut db, 1) {
        Ok(c) => c,
        Err(e) => panic!("{}", e),
    };
    let user = match User::create(&mut db, &username, &password, clearance) {
        Ok(u) => u,
        Err(e) => panic!("{}", e),
    };
    println!("Created new user\r\nusername: {}\r\nhash: {}", user.get_name(), user.get_password_hash());
}
