use migaton::traits::Migrations;
pub struct RustersMigrator;
impl RustersMigrator {
    const MIGRATIONS_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/sql/migrations");
}
impl Migrations for RustersMigrator {
    fn get_mig_path() -> &'static str {
        return Self::MIGRATIONS_PATH;
    }
}
