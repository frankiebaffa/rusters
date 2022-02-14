use {
    chrono::Duration,
    crate::{
        Clearance,
        ConsumableToken,
        Consumer,
        RustersMigrator,
        Session,
        SessionCookie,
        User,
    },
    migaton::traits::DoMigrations,
    serial_test::serial,
    std::path::PathBuf,
    worm::{
        core::{
            DbCtx,
            DbContext,
            UniqueName,
        },
        derive::WormDb,
    },
};
fn delete_db_file_if_exists() {
    let db_path = PathBuf::from("./RustersDb.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).unwrap();
    }
}
#[derive(WormDb)]
#[db(var(name="RUSTERSDBS"))]
struct Database {
    context: DbContext,
}
fn get_db_ctx() -> Database {
    let mut db = Database::init();
    db.context.attach_dbs();
    return db;
}
fn get_clearance(db: &mut Database) -> Clearance {
    let name = "King";
    let c_res = Clearance::from_name(db, name);
    assert!(c_res.is_ok());
    let c = c_res.unwrap();
    assert_eq!(c.get_name(), name);
    return c;
}
#[test]
#[serial]
fn get_clearance_by_name() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    get_clearance(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
fn get_token_type(db: &mut Database) -> Consumer {
    let type_res = Consumer::get_or_create(db, "create_user");
    assert!(type_res.is_ok());
    let t = type_res.unwrap();
    assert_eq!(t.get_name(), "create_user");
    t
}
fn get_token(db: &mut Database) -> (ConsumableToken, String) {
    let t_type = get_token_type(db);
    let t_res = ConsumableToken::create_new(db, t_type, Duration::hours(1));
    assert!(t_res.is_ok());
    let (t, h) = t_res.unwrap();
    assert!(!h.is_empty());
    return (t, h);
}
#[test]
#[serial]
fn create_consumable_token() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    get_token(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
fn get_new_user(db: &mut Database) -> (String, User) {
    let username = "test_user_1";
    let password = "$this_is_a_password_1";
    let c = get_clearance(db);
    let u_res = User::create(db, username, password, c);
    assert!(u_res.is_ok());
    let u = u_res.unwrap();
    assert_eq!(u.get_name(), username);
    return (password.to_string(), u);
}
#[test]
#[serial]
fn create_new_user() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    get_new_user(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
fn get_session(db: &mut Database) -> (Session, String) {
    let s_res = Session::create_new(db);
    assert!(s_res.is_ok());
    return s_res.unwrap();
}
#[test]
#[serial]
fn create_session() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    get_session(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
fn check_session(db: &mut Database) -> (Session, String) {
    let (_, h) = get_session(db);
    let s_res = Session::get_active(db, &h);
    assert!(s_res.is_ok());
    let s = s_res.unwrap();
    let h_res = s.get_hash(db);
    assert!(h_res.is_ok());
    let h2 = h_res.unwrap();
    assert_eq!(h, h2);
    return (s, h2);
}
#[test]
#[serial]
fn create_and_check_session() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    check_session(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
fn check_user_logged_in(db: &mut Database, s: &Session) -> bool {
    let u_opt_res = s.is_logged_in(db);
    assert!(u_opt_res.is_ok());
    let u_opt = u_opt_res.unwrap();
    return u_opt.is_some();
}
fn do_login(db: &mut Database, s: &Session, u: &User, p: &str) -> User {
    let n = u.get_name();
    let u2_res = s.login(db, &n, p);
    assert!(u2_res.is_ok());
    let u2 = u2_res.unwrap();
    assert_eq!(u.get_name(), u2.get_name());
    return u2;
}
fn create_user_and_login(db: &mut Database) -> (Session, User) {
    let (s, _) = check_session(db);
    let l1 = check_user_logged_in(db, &s);
    assert!(!l1);
    let (p, u) = get_new_user(db);
    do_login(db, &s, &u, &p);
    let l2 = check_user_logged_in(db, &s);
    assert!(l2);
    return (s, u);
}
#[test]
#[serial]
fn login_user() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    create_user_and_login(&mut db);
    RustersMigrator::migrate_down::<Database>(None);
}
#[test]
#[serial]
fn logout_user() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    let (s, _) = create_user_and_login(&mut db);
    let o_res = s.log_out(&mut db);
    assert!(o_res.is_ok());
    let o = o_res.unwrap();
    assert!(o);
    assert!(!check_user_logged_in(&mut db, &s));
    RustersMigrator::migrate_down::<Database>(None);
}
const COOKIE_KEY: &'static str = "Hello";
fn create_cookie(db: &mut Database, s: &Session) -> (String, String) {
    let key = COOKIE_KEY;
    let val = "World";
    let c_res = s.set_cookie(db, key, val);
    assert!(c_res.is_ok());
    let c = c_res.unwrap();
    assert_eq!(c.get_name(), key);
    assert_eq!(c.get_value(), val);
    return (key.to_string(), val.to_string());
}
#[test]
#[serial]
fn set_cookie() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    let (s, _) = get_session(&mut db);
    create_cookie(&mut db, &s);
    RustersMigrator::migrate_down::<Database>(None);
}
fn check_cookie(db: &mut Database, s: &Session) -> SessionCookie {
    let c1_opt_res = s.read_cookie(db, COOKIE_KEY);
    assert!(c1_opt_res.is_ok());
    let c1_opt = c1_opt_res.unwrap();
    assert!(c1_opt.is_none());
    let (k, v) = create_cookie(db, &s);
    let c2_opt_res = s.read_cookie(db, &k);
    assert!(c2_opt_res.is_ok());
    let c2_opt = c2_opt_res.unwrap();
    assert!(c2_opt.is_some());
    let c2 = c2_opt.unwrap();
    assert_eq!(c2.get_name(), k);
    assert_eq!(c2.get_value(), v);
    return c2;
}
#[test]
#[serial]
fn read_cookie() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    let (s, _) = get_session(&mut db);
    check_cookie(&mut db, &s);
    RustersMigrator::migrate_down::<Database>(None);
}
#[test]
#[serial]
fn delete_cookie() {
    delete_db_file_if_exists();
    RustersMigrator::migrate_up::<Database>(None);
    let mut db = get_db_ctx();
    let (s, _) = get_session(&mut db);
    let c = check_cookie(&mut db, &s);
    let d_res = s.delete_cookie(&mut db, &c.get_name());
    assert!(d_res.is_ok());
    let d = d_res.unwrap();
    assert!(d);
    let c_opt_res = s.read_cookie(&mut db, &c.get_name());
    assert!(c_opt_res.is_ok());
    let c_opt = c_opt_res.unwrap();
    assert!(c_opt.is_none());
    RustersMigrator::migrate_down::<Database>(None);
}
// TODO: delete after testing!
#[test]
fn migrate_up_test() {
    RustersMigrator::migrate_up::<Database>(None);
}
