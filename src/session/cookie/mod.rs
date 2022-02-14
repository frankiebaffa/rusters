use {
    chrono::{
        DateTime,
        Utc,
    },
    crate::session::Session,
    worm::derive::Worm,
};
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb",name="SessionCookies",alias="sessioncookie"))]
pub struct SessionCookie {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Session_PK", foreign_key="Session", insertable))]
    session_pk: i64,
    #[dbcolumn(column(name="Name", insertable, unique_name))]
    name: String,
    #[dbcolumn(column(name="Value", insertable))]
    value: String,
    #[dbcolumn(column(name="Active", active_flag))]
    active: bool,
    #[dbcolumn(column(name="Created_DT", insertable, utc_now))]
    created_dt: DateTime<Utc>,
}
