use {
    migaton::traits::DoMigrations,
    rusters::RustersMigrator,
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
    let mut mem_db = Database::init();
    mem_db.context.attach_temp_dbs();
    let mut db = Database::init();
    db.context.attach_dbs();
    let skips = RustersMigrator::migrate_up(&mut mem_db, &mut db);
    println!("{} migrations were skipped", skips);
}
