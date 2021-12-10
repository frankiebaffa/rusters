use {
    chrono::{
        DateTime,
        Utc,
    },
    worm::{
        core::DbCtx,
        derive::Worm,
    },
};
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="TokenTypes", alias="tokentypes"))]
pub struct TokenType {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Name"))]
    name: String,
    #[dbcolumn(column(name="Description"))]
    description: String,
    #[dbcolumn(column(name="Created_DT"))]
    created_dt: DateTime<Utc>,
}
