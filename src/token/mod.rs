pub mod tokentype;
pub mod createuser;
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
        hash::Hasher,
    },
    tokentype::TokenType,
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
    #[dbcolumn(column(name="TokenType_PK", foreign_key="TokenType", insertable))]
    tokentype_pk: i64,
    #[dbcolumn(column(name="Hash", insertable))]
    hash: String,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
    #[dbcolumn(column(name="Expired_DT", insertable))]
    expired_dt: DateTime<Utc>,
}
impl Token {
    pub fn generate_for_new_user(
        db: &mut impl DbCtx
    ) -> Result<Token, RustersError> {
        let hash = Hasher::get_token_hash()?;
        let token_type = Query::<TokenType>::select()
            .where_eq::<TokenType>(TokenType::NAME, &"CreateUser")
            .execute_row(db)
            .quick_match()?;
        return Token::insert_new(
            db,
            token_type.get_id(),
            hash, Utc::now(),
            Utc::now() + Duration::days(1)
        ).quick_match();
    }
    pub fn generate_for_new_session(
        db: &mut impl DbCtx
    ) -> Result<Token, RustersError> {
        let hash = Hasher::get_token_hash()?;
        let token_type = Query::<TokenType>::select()
            .where_eq::<TokenType>(TokenType::NAME, &"Session")
            .execute_row(db)
            .quick_match()?;
        return Token::insert_new(
            db,
            token_type.get_id(),
            hash, Utc::now(),
            Utc::now() + Duration::days(1)
        ).quick_match();
    }
}
