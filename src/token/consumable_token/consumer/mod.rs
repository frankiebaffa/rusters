use {
    buildlite::Query,
    chrono::{
        DateTime,
        Utc,
    },
    crate::error::{
        MatchRustersError,
        RustersError,
    },
    worm::{
        core::DbCtx,
        derive::Worm,
    },
};
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Consumers", alias="consumer"))]
pub struct Consumer {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Name", unique_name, insertable))]
    name: String,
    #[dbcolumn(column(name="IsActive", active_flag))]
    is_active: bool,
    #[dbcolumn(column(name="Created_DT", insertable, utc_now))]
    created_dt: DateTime<Utc>,
}
impl Consumer {
    pub fn get_or_create(
        db: &mut impl DbCtx, name: impl AsRef<str>
    ) -> Result<Self, RustersError> {
        let n = name.as_ref();
        let existing = Query::<Consumer>::select()
            .where_eq::<Consumer>(Consumer::NAME, &n)
            .execute(db)
            .quick_match()?;
        if existing.len() > 0 {
            Ok(existing.into_iter().next().unwrap())
        } else {
            Consumer::insert_new(db, n.to_string()).quick_match()
        }
    }
}
