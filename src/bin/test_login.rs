use {
    rusters::Session,
    std::io::BufRead,
    worm::{
        core::{
            DbCtx,
            DbContext,
            UniqueName
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
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let session = match Session::get_active(&mut db, &session_hash) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    let user_opt = match session.is_logged_in(&mut db) {
        Ok(u) => u,
        Err(e) => panic!("{}", e),
    };
    if user_opt.is_some() {
        let user = user_opt.unwrap();
        println!("Session already logged in as {}", user.get_name());
        return;
    } else {
        println!("Not yet logged in");
    }
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
    let user = match session.login(&mut db, &username, &password) {
        Ok(u) => u,
        Err(e) => panic!("{}", e),
    };
    println!("Logged in user\r\nusername: {}\r\nsession hash: {}", user.get_name(), &session_hash);
}
