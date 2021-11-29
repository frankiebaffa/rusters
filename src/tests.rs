use {
    crate::{
        Clearance,
        CreateUserToken,
        RustersMigrator,
        Session,
        SessionCookie,
        User,
    },
    migaton::traits::DoMigrations,
    serial_test::serial,
    worm::{
        core::{
            DbCtx,
            DbContext,
            UniqueName,
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
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    get_clearance(&mut db);
    migrate_down(&mut mem_db, &mut db);
}
fn get_token(db: &mut Database) -> String {
    let c = get_clearance(db);
    let t_res = CreateUserToken::create_new(db, c);
    assert!(t_res.is_ok());
    let (_, h) = t_res.unwrap();
    assert!(!h.is_empty());
    return h;
}
#[test]
#[serial]
fn create_user_creation_token() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    get_token(&mut db);
    migrate_down(&mut mem_db, &mut db);
}
fn get_new_user(db: &mut Database) -> (String, User) {
    let h = get_token(db);
    let v_res = CreateUserToken::token_valid(db, &h);
    assert!(v_res.is_ok());
    let username = "test_user_1";
    let password = "$this_is_a_password_1";
    let u_res = CreateUserToken::use_token(db, &h, username, password);
    assert!(u_res.is_ok());
    let u = u_res.unwrap();
    assert_eq!(u.get_name(), username);
    return (password.to_string(), u);
}
#[test]
#[serial]
fn create_new_user() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    get_new_user(&mut db);
    migrate_down(&mut mem_db, &mut db);
}
fn get_session(db: &mut Database) -> (Session, String) {
    let s_res = Session::create_new(db);
    assert!(s_res.is_ok());
    return s_res.unwrap();
}
#[test]
#[serial]
fn create_session() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    get_session(&mut db);
    migrate_down(&mut mem_db, &mut db);
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
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    check_session(&mut db);
    migrate_down(&mut mem_db, &mut db);
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
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    create_user_and_login(&mut db);
    migrate_down(&mut mem_db, &mut db);
}
#[test]
#[serial]
fn logout_user() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    let (s, _) = create_user_and_login(&mut db);
    let o_res = s.log_out(&mut db);
    assert!(o_res.is_ok());
    let o = o_res.unwrap();
    assert!(o);
    assert!(!check_user_logged_in(&mut db, &s));
    migrate_down(&mut mem_db, &mut db);
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
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    let (s, _) = get_session(&mut db);
    create_cookie(&mut db, &s);
    migrate_down(&mut mem_db, &mut db);
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
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
    let (s, _) = get_session(&mut db);
    check_cookie(&mut db, &s);
    migrate_down(&mut mem_db, &mut db);
}
#[test]
#[serial]
fn delete_cookie() {
    let (mut mem_db, mut db) = get_db_ctx();
    migrate_up(&mut mem_db, &mut db);
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
    migrate_down(&mut mem_db, &mut db);
}
