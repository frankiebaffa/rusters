use {
    rusters::RustersMigrator,
    sqlx::sqlite::SqlitePool,
};
#[async_std::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let db_path = std::env::var("DATABASE_URL").unwrap();
    std::fs::File::create(&db_path.replace("sqlite://", "")).unwrap();
    let db = SqlitePool::connect(&db_path).await.unwrap();
    RustersMigrator::migrate(&db).await.unwrap();
}
