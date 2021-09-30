use {
    rusters::context::Context,
    migaton::Migrator,
    rusqlite::{
        Connection,
        Error,
    },
};
fn main() -> Result<(), Error> {
    let ctx = Context::init().unwrap();
    println!("{}", ctx.db_name);
    let mut mem_c = Connection::open(":memory:")?;
    mem_c.execute(&format!("attach ':memory:' as {}", ctx.db_name), [])?;
    let mut c = Connection::open(":memory:")?;
    println!("{}", ctx.db_path);
    c.execute(&format!("attach '{}' as {}", ctx.get_path_to_db(), ctx.db_name), [])?;
    let skips = match Migrator::do_up(&mut mem_c, &mut c, &ctx.migration_path) {
        Ok(res) => res,
        Err(e) => panic!("{}", e),
    };
    println!("{} migrations were skipped", skips);
    return Ok(());
}
