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
        user::{
            clearance::Clearance,
            User,
        },
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
#[dbmodel(table(schema="RustersDb", name="CreateUserTokens", alias="createusertokens"))]
pub struct CreateUserToken {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Token_PK", foreign_key="Token", insertable))]
    token_pk: i64,
    #[dbcolumn(column(name="Clearance_PK", foreign_key="Clearance", insertable))]
    clearance_pk: i64,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl CreateUserToken {
    pub fn create_new(
        db: &mut impl DbCtx,
        clearance: Clearance
    ) -> Result<(CreateUserToken, String), RustersError> {
        let token = Token::generate_for_new_user(db)?;
        let cut = CreateUserToken::insert_new(
            db,
            token.get_id(),
            clearance.get_id(),
            Utc::now()
        ).quick_match()?;
        return Ok((cut, token.get_hash()));
    }
    pub fn token_valid<'a>(
        db: &mut impl DbCtx,
        hash: &'a str,
    ) -> Result<CreateUserToken, RustersError> {
        return Query::<CreateUserToken>::select()
            .join_fk::<Token>().join_and()
            .join_fk_eq::<Token>(Token::HASH, &hash).join_and()
            .join_fk_gt::<Token>(Token::EXPIRED_DT, &Utc::now())
            .execute_row(db)
            .quick_match();
    }
    pub fn use_token<'a>(
        db: &mut impl DbCtx,
        hash: &'a str,
        username: &'a str,
        password: &'a str,
    ) -> Result<User, RustersError> {
        let token = Self::token_valid(db, hash)?;
        let safe_now = Utc::now() - Duration::seconds(-1);
        let clearance = Query::<Clearance>::select()
            .where_eq::<Clearance>(Clearance::PRIMARY_KEY, &token.clearance_pk)
            .execute_row(db)
            .quick_match()?;
        // force expire token
        Query::<Token>::update()
            .set(Token::EXPIRED_DT, &safe_now)
            .where_eq::<Token>(Token::PRIMARY_KEY, &token.token_pk)
            .execute_update(db)
            .quick_match()?;
        return User::create(db, username, password, clearance);
    }
}
