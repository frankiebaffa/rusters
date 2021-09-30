use std::env;
pub struct Context {
    pub db_path: String,
    pub db_name: String,
    pub migration_path: String,
}
impl Context {
    pub fn init() -> Result<Context, env::VarError> {
        match dotenv::dotenv() {
            Ok(_) => {},
            Err(_) => {},
        }
        let db_path = env::var("RUSTERS_DB_PATH")?;
        let migration_path = env::var("RUSTERS_MIGRATION_PATH")?;
        return Ok(Context {
            db_name: String::from("RustersDb"),
            db_path,
            migration_path,
        });
    }
    pub fn get_path_to_db(&self) -> String {
        return if self.db_path.ends_with("/") {
            format!("{}{}.db", self.db_path, self.db_name)
        } else {
            format!("{}/{}.db", self.db_path, self.db_name)
        }
    }
}
