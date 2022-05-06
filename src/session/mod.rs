pub mod cookie;
use {
    chrono::{
        DateTime,
        Utc,
    },
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        token::Token,
    },
    sqlx::{ FromRow, SqlitePool, query_as, query, },
};
#[derive(FromRow)]
pub struct Session {
    pk: i64,
    token_pk: i64,
    created_dt: DateTime<Utc>,
}
impl Session {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_token_pk(&self) -> i64 {
        self.token_pk
    }
    pub fn get_created_dt(&self) -> DateTime<Utc> {
        self.created_dt
    }
    pub async fn lookup<'a>(
        db: &SqlitePool, token: &Token
    ) -> Result<Session, RustersError> {
        query_as::<_, Session>("
            select
                pk,
                token_pk,
                created_dt
            from Sessions as s
            where s.token_pk = $1"
        ).bind(token.get_hash())
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn lookup_by_pk<'a>(
        db: &SqlitePool, pk: i64
    ) -> Result<Session, RustersError> {
        query_as::<_, Session>("
            select
                pk,
                token_pk,
                created_dt
            from Sessions as s
            where s.pk = $1"
        ).bind(pk)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn insert_new(
        db: &SqlitePool, token: &Token
    ) -> Result<Self, RustersError> {
        let pk = query("
            insert into Sessions (
                token_pk,
                created_dt
            ) values (
                $1,
                $2
            )"
        ).bind(token.get_pk())
            .bind(Utc::now())
            .execute(db)
            .await
            .quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
}
