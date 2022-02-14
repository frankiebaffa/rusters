pub mod tokentype;
pub mod consumable_token;
use {
    buildlite::Query,
    chrono::{
        DateTime,
        Duration,
        Utc,
    },
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        hash::Hash,
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
#[dbmodel(table(schema="RustersDb", name="Tokens", alias="tokens"))]
pub struct Token {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Hash", insertable))]
    hash: String,
    #[dbcolumn(column(name="Created_DT", insertable, utc_now))]
    created_dt: DateTime<Utc>,
    #[dbcolumn(column(name="Expired_DT", insertable))]
    expired_dt: DateTime<Utc>,
}
impl Token {
    /// Creates a hash from a random uuid
    pub fn from_hash(db: &mut impl DbCtx, hash: impl Hash, exp: Duration) -> Result<Self, RustersError> {
        Token::insert_new(
            db,
            hash.get_hash(),
            Utc::now() + exp,
        ).quick_match()
    }
    /// Forces a token to expire
    pub fn force_expire(&self, db: &mut impl DbCtx) -> Result<usize, RustersError> {
        let safe_now = Utc::now() - Duration::seconds(-1);
        Query::<Token>::update()
            .set(Token::EXPIRED_DT, &safe_now)
            .where_eq::<Token>(Token::PRIMARY_KEY, &self.pk)
            .execute_update(db)
            .quick_match()
    }
}
