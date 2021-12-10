use {
    buildlite::Query,
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
#[dbmodel(table(schema="RustersDb", name="Clearances", alias="clearance"))]
pub struct Clearance {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Sequence"))]
    sequence: i64,
    #[dbcolumn(column(name="Name"))]
    name: String,
}
impl Clearance {
    pub fn retrieve_all(db: &mut impl DbCtx) -> Result<Vec<Self>, RustersError> {
        return Query::<Self>::select()
            .orderby_asc(Self::SEQUENCE)
            .execute(db)
            .quick_match();
    }
    pub fn from_name<'a>(
        db: &mut impl DbCtx, name: &'a str
    ) -> Result<Self, RustersError> {
        return Query::<Self>::select()
            .where_eq::<Self>(Self::NAME, &name)
            .execute_row(db)
            .quick_match();
    }
}
