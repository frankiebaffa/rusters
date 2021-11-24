use {
    rusters::Session,
    worm::{
        core::{
            DbCtx,
            DbContext,
        },
        derive::WormDb,
    },
};
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
    let hash = match session.get_hash(&mut db) {
        Ok(h) => h,
        Err(e) => panic!("{}", e),
    };
    if session_hash.eq(&hash) {
        println!("Resumed session!")
    } else {
        println!("Error, created new session");
    }
}
