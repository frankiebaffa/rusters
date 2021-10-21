use {
    rusters::{
        Clearance,
        CreateUserToken,
        MatchRustersError,
        RustersError,
    },
    std::io::BufRead,
    worm::{
        DbContext,
        DbCtx,
    },
    worm_derive::WormDb,
};
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
fn main() -> Result<(), RustersError> {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut db = Database::init();
    db.context.attach_dbs();
    let clearances = Clearance::retrieve_all(&mut db)?;
    println!("Pick a clearance by number");
    clearances.iter().for_each(|c| {
        println!("{}: {}", c.get_sequence(), c.get_name());
    });
    let mut choice = String::new();
    lock.read_line(&mut choice).quick_match()?;
    choice = choice.trim().to_string();
    let choice_i = match choice.parse::<i64>() {
        Ok(i) => i,
        Err(e) => panic!("{}", e),
    };
    let clearance = clearances.into_iter().filter(|c| c.get_sequence().eq(&choice_i))
        .nth(0)
        .unwrap();
    let (_, hash) = CreateUserToken::create_new(&mut db, clearance)?;
    println!("Token hash: {}", hash);
    Ok(())
}
