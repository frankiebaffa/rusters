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
        hash::{
            Basic,
            Hash,
        },
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
    pub fn generate_for_new_user(
        db: &mut impl DbCtx
    ) -> Result<Token, RustersError> {
        let hash = Basic::rand()?;
        return Token::insert_new(
            db,
            hash.hash,
            Utc::now() + Duration::days(1)
        ).quick_match();
    }
    pub fn generate_for_new_session(
        db: &mut impl DbCtx
    ) -> Result<Token, RustersError> {
        let hash = Hasher::get_token_hash()?;
        return Token::insert_new(
            db,
            hash,
            Utc::now() + Duration::days(1)
        ).quick_match();
    }
    pub fn force_expire(&self, db: &mut impl DbCtx) -> Result<usize, RustersError> {
        let safe_now = Utc::now() - Duration::seconds(-1);
        Query::<Token>::update()
            .set(Token::EXPIRED_DT, &safe_now)
            .where_eq::<Token>(Token::PRIMARY_KEY, &self.pk)
            .execute_update(db)
            .quick_match()
    }
}
