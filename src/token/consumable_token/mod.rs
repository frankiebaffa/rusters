pub mod consumer;
use {
    chrono::{
        DateTime,
        Utc,
    },
    consumer::Consumer,
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        token::Token,
    },
    sqlx::{ FromRow, SqlitePool, query, query_as, },
};
#[derive(FromRow)]
pub struct ConsumableToken {
    pk: i64,
    token_pk: i64,
    consumer_pk: i64,
    created_dt: DateTime<Utc>,
}
impl ConsumableToken {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_token_pk(&self) -> i64 {
        self.token_pk
    }
    pub fn get_consumer_pk(&self) -> i64 {
        self.consumer_pk
    }
    pub fn get_created_dt(&self) -> DateTime<Utc> {
        self.created_dt
    }
    pub async fn lookup_by_pk(
        db: &SqlitePool, pk: i64
    ) -> Result<Self, RustersError> {
        query_as::<_, Self>("
            select
                PK,
                Token_PK,
                Consumer_PK,
                Created_DT
            from ConsumableTokens
            where PK = $1"
        ).bind(pk)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn insert_new(
        db: &SqlitePool, token: Token, consumer: Consumer
    ) -> Result<Self, RustersError> {
        let pk = query("
            insert into ConsumableTokens (
                Token_PK,
                Consumer_PK,
                Created_DT
            ) values (
                $1,
                $2,
                $3
            )"
        ).bind(token.get_pk())
            .bind(consumer.get_pk())
            .bind(Utc::now())
            .execute(db)
            .await
            .quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
}
