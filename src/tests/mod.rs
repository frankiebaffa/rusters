use {
    crate::{
        Basic,
        ConsumableToken,
        Consumer,
        Hash,
        RustersMigrator,
        Session,
        SessionCookie,
        Token,
        User,
    },
    sqlx::SqlitePool,
    std::path::PathBuf,
};
fn get_file_name() -> String {
    let hash_res = Basic::rand();
    let hash = hash_res.unwrap();
    hash.get_hash()
}
fn create_db_file_if_not_exist<'a>(name: &'a str) {
    let db_path = PathBuf::from(
        format!("./test_dbs/{}.db", name)
    );
    if !db_path.exists() {
        let file_res = std::fs::File::create(db_path);
        file_res.unwrap();
    }
}
fn delete_db_file_if_exists<'a>(name: &'a str) {
    let db_path = PathBuf::from(
        format!("./test_dbs/{}.db", name)
    );
    if db_path.exists() {
        let rem_res = std::fs::remove_file(db_path);
        rem_res.unwrap();
    }
}
async fn get_db<'a>(name: &'a str) -> SqlitePool {
    let path = format!("sqlite://./test_dbs/{}.db", name);
    SqlitePool::connect(&path).await.unwrap()
}
#[async_std::test]
async fn migrate_up() {
    let db_name = get_file_name();
    create_db_file_if_not_exist(&db_name);
    let db = get_db(&db_name).await;
    RustersMigrator::migrate(&db).await.unwrap();
    delete_db_file_if_exists(&db_name);
}
async fn get_token(db: &SqlitePool) -> Token {
    let token_res = Token::insert_basic(db, None).await;
    token_res.unwrap()
}
#[async_std::test]
async fn create_token() {
    let db_name = get_file_name();
    create_db_file_if_not_exist(&db_name);
    let db = get_db(&db_name).await;
    RustersMigrator::migrate(&db).await.unwrap();
    get_token(&db).await;
    delete_db_file_if_exists(&db_name);
}
//async fn get_consumer(db: &SqlitePool) -> Consumer {
//    let consumer_res = Consumer::get_or_create(db, "create_user").await;
//    let t = consumer_res.unwrap();
//    assert_eq!(t.get_name(), "create_user");
//    t
//}
//async fn get_consumable_token(db: &SqlitePool, c: Consumer, t: Token) -> ConsumableToken {
//    let c_token_res = ConsumableToken::insert_new(db, t, c).await;
//    c_token_res.unwrap()
//}
//#[async_std::test]
//async fn create_consumable_token() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let c = get_consumer(&db).await;
//    let t = get_token(&db).await;
//    let _ = get_consumable_token(&db, c, t).await;
//    delete_db_file_if_exists(&db_name);
//}
//async fn get_new_user(db: &SqlitePool) -> (String, User) {
//    let username = "test_user_1";
//    let password = "$this_is_a_password_1";
//    let u_res = User::insert_new(db, username, password).await;
//    let u = u_res.unwrap();
//    assert_eq!(u.get_username(), username);
//    return (password.to_string(), u);
//}
//#[async_std::test]
//async fn create_new_user() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    get_new_user(&db).await;
//    delete_db_file_if_exists(&db_name);
//}
//async fn get_session(db: &SqlitePool, t: &Token) -> Session {
//    let s_res = Session::insert_new(db, t).await;
//    s_res.unwrap()
//}
//#[async_std::test]
//async fn create_session() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    get_session(&db, &t).await;
//    delete_db_file_if_exists(&db_name);
//}
//#[async_std::test]
//async fn create_and_check_session() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    assert_eq!(t.get_pk(), s.get_token_pk());
//    delete_db_file_if_exists(&db_name);
//}
//async fn check_user_logged_in(db: &SqlitePool, s: &Session) -> bool {
//    let c_opt = SessionCookie::read(db, s, SessionCookie::LOGIN_COOKIE)
//        .await
//        .unwrap();
//    return c_opt.is_some();
//}
//async fn do_login(db: &SqlitePool, s: &Session, u: &User, p: &str) {
//    let u2_res = User::lookup_by_credentials(db, &u.get_username(), p).await;
//    let u2 = u2_res.unwrap();
//    let c1_res = SessionCookie::login(db, s, &u2).await;
//    let c1 = c1_res.unwrap();
//    assert_eq!(c1.get_name(), SessionCookie::LOGIN_COOKIE);
//}
//async fn create_user_and_login(db: &SqlitePool, s: &Session) -> User {
//    let is_1 = SessionCookie::has_login_cookie(db, s).await.unwrap();
//    assert!(!is_1);
//    let (p, u) = get_new_user(db).await;
//    do_login(db, &s, &u, &p).await;
//    let l2 = check_user_logged_in(db, &s).await;
//    assert!(l2);
//    u
//}
//#[async_std::test]
//async fn login_user() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    create_user_and_login(&db, &s).await;
//    delete_db_file_if_exists(&db_name);
//}
//#[async_std::test]
//async fn logout_user() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    let _ = create_user_and_login(&db, &s).await;
//    assert!(check_user_logged_in(&db, &s).await);
//    SessionCookie::logout(&db, &s).await.unwrap();
//    assert!(!check_user_logged_in(&db, &s).await);
//    delete_db_file_if_exists(&db_name);
//}
//const COOKIE_KEY: &'static str = "Hello";
//async fn create_cookie(db: &SqlitePool, s: &Session) -> (String, String) {
//    let key = COOKIE_KEY;
//    let val = "World";
//    let c_res = SessionCookie::set(db, s, key, val).await;
//    let c = c_res.unwrap();
//    assert_eq!(c.get_name(), key);
//    assert_eq!(c.get_value(), val);
//    return (key.to_string(), val.to_string());
//}
//#[async_std::test]
//async fn set_cookie() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let mut db = &get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    create_cookie(&mut db, &s).await;
//    delete_db_file_if_exists(&db_name);
//}
//async fn check_cookie(db: &SqlitePool, s: &Session) -> SessionCookie {
//    let c1_opt_res = SessionCookie::read(db, s, COOKIE_KEY).await;
//    let c1_opt = c1_opt_res.unwrap();
//    assert!(c1_opt.is_none());
//    let (k, v) = create_cookie(db, &s).await;
//    let c2_opt_res = SessionCookie::read(db, &s, &k).await;
//    let c2_opt = c2_opt_res.unwrap();
//    assert!(c2_opt.is_some());
//    let c2 = c2_opt.unwrap();
//    assert_eq!(c2.get_name(), k);
//    assert_eq!(c2.get_value(), v);
//    return c2;
//}
//#[async_std::test]
//async fn read_cookie() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    check_cookie(&db, &s).await;
//    delete_db_file_if_exists(&db_name);
//}
//#[async_std::test]
//async fn delete_cookie() {
//    let db_name = get_file_name();
//    create_db_file_if_not_exist(&db_name);
//    let mut db = get_db(&db_name).await;
//    RustersMigrator::migrate(&db).await.unwrap();
//    let t = get_token(&db).await;
//    let s = get_session(&db, &t).await;
//    let c = check_cookie(&db, &s).await;
//    let d_res = SessionCookie::delete(&db, &s, &c.get_name()).await;
//    d_res.unwrap();
//    let c_opt_res = SessionCookie::read(&mut db, &s, &c.get_name()).await;
//    let c_opt = c_opt_res.unwrap();
//    assert!(c_opt.is_none());
//    delete_db_file_if_exists(&db_name);
//}
