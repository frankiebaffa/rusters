pub mod consumer;
use {
    buildlite::Query,
    chrono::{
        DateTime,
        Duration,
        Utc,
    },
    consumer::Consumer,
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        hash::{ Basic, Hash, },
        token::Token,
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
#[dbmodel(table(schema="RustersDb", name="ConsumableTokens", alias="consumable_token"))]
pub struct ConsumableToken {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Token_PK", foreign_key="Token", insertable))]
    token_pk: i64,
    #[dbcolumn(column(name="Consumer_PK", foreign_key="Consumer", insertable))]
    consumer_pk: i64,
    #[dbcolumn(column(name="Created_DT", insertable, utc_now))]
    created_dt: DateTime<Utc>,
}
impl ConsumableToken {
    pub fn create_new(
        db: &mut impl DbCtx,
        consumer: Consumer,
        exp: Duration,
    ) -> Result<(Self, String), RustersError> {
        let hash = Basic::rand()?;
        let token = Token::from_hash(db, hash, exp)?;
        let cut = Self::insert_new(
            db,
            token.get_id(),
            consumer.get_id(),
        ).quick_match()?;
        return Ok((cut, token.get_hash()));
    }
    pub fn can_consume<'a>(
        db: &mut impl DbCtx,
        hash: impl AsRef<str>,
        consumer_name: impl AsRef<str>,
    ) -> Result<Self, RustersError> {
        let h = hash.as_ref();
        let cn = consumer_name.as_ref();
        return Query::<Self>::select()
            .join_fk::<Token>().join_and()
            .join_fk_eq::<Token>(Token::HASH, &h).join_and()
            .join_fk_gt::<Token>(Token::EXPIRED_DT, &Utc::now())
            .join_fk::<Consumer>().join_and()
            .join_fk_eq::<Consumer>(Consumer::NAME, &cn)
            .execute_row(db)
            .quick_match();
    }
    pub fn consume<'a>(
        &self,
        db: &mut impl DbCtx,
    ) -> Result<bool, RustersError> {
        let token = Query::<Token>::select()
            .where_eq::<Token>(Token::PK, &self.token_pk)
            .execute_row(db)
            .quick_match()?;
        match token.force_expire(db) {
            Ok(row_alt) => {
                if row_alt > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            Err(e) => Err(e),
        }
    }
}
