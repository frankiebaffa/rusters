use rusters::CreateUserToken;
use rusters::RustersError;
use worm::core::{DbCtx, DbContext};
use worm::core::traits::uniquename::UniqueName;
use worm::derive::WormDb;
use std::io::BufRead;
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
fn main() -> Result<(), RustersError> {
    match dotenv::dotenv() {
        _ => {}
    }
    let create_hash = std::env::var("CREATE_USER_HASH").unwrap();
    let mut db = Database::init();
    db.context.attach_dbs();
    CreateUserToken::token_valid(&mut db, &create_hash)?;
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
    let user = CreateUserToken::use_token(&mut db, &create_hash, &username, &password)?;
    println!("Created new user\r\nusername: {}\r\nhash: {}", user.get_name(), user.get_password_hash());
    Ok(())
}
