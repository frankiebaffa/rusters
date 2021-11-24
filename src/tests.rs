use {
    crate::{
        Clearance,
        RustersMigrator,
    },
    migaton::traits::DoMigrations,
    serial_test::serial,
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
fn get_db_ctx() -> (Database, Database) {
    let mut mem_db = Database::init();
    mem_db.context.attach_temp_dbs();
    let mut db = Database::init();
    db.context.attach_dbs();
    return (mem_db, db);
}
fn migrate_up(mem_db: &mut Database, db: &mut Database) {
    RustersMigrator::migrate_up(mem_db, db);
}
fn migrate_down(mem_db: &mut Database, db: &mut Database) {
    RustersMigrator::migrate_down(mem_db, db);
}
#[test]
#[serial]
fn get_clearance_by_name() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    migrate_down(&mut mem_db, &mut db);
}
#[test]
#[serial]
fn create_user_token_and_user() {

}
