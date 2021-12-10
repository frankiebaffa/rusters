pub mod clearance;
use {
    buildlite::Query,
    chrono::{
        DateTime,
        Utc,
    },
    clearance::Clearance,
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        hash::Hasher,
    },
    worm::{
        core::{
            DbCtx,
            PrimaryKey,
        },
        derive::Worm,
    },
};
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Users", alias="user"))]
pub struct User {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Username", insertable, unique_name))]
    username: String,
    #[dbcolumn(column(name="PasswordHash", insertable))]
    password_hash: String,
    #[dbcolumn(column(name="Salt", insertable))]
    salt: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Clearance_PK", insertable, foreign_key="Clearance"))]
    clearance_pk: i64,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl User {
    pub fn create<'a>(db: &mut impl DbCtx, username: &'a str, password: &'a str, clearance: Clearance) -> Result<Self, RustersError> {
        let hashed = Hasher::hash_password(password.to_owned())?;
        let salt = hashed.salt;
        let pw_hash = hashed.b64_hash;
        let now = Utc::now();
        let user = User::insert_new(db, username.to_owned(), pw_hash, salt, clearance.get_id(), now).quick_match()?;
        return Ok(user);
    }
    pub fn get_clearance_level<'a>(&self, db: &mut impl DbCtx) -> Result<i64, RustersError> {
        let c = Query::<Clearance>::select()
            .join::<User>().join_and()
            .join_eq::<User>(User::PK, &self.pk)
            .execute_row(db)
            .quick_match()?;
        return Ok(c.get_sequence());
    }
}
