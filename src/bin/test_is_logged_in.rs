use rusters::Session;
use worm::core::{DbCtx, DbContext};
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
