use {
    rusters::Session,
    worm::{
        DbCtx,
        DbContext,
    },
    worm_derive::WormDb,
};
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
fn main() {
    let mut db = Database::init();
    db.context.attach_dbs();
    let (_, hash) = match Session::create_new(&mut db) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    println!("Session created! hash: {}", hash);
}
