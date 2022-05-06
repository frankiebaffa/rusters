pub mod consumable_token;
use {
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
            Secure,
        },
    },
    sqlx::{ FromRow, SqlitePool, query, query_as, },
};
#[derive(FromRow)]
pub struct Token {
    pk: i64,
    hash: String,
    created_dt: DateTime<Utc>,
    expired_dt: DateTime<Utc>,
}
impl Token {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }
    pub fn get_created_dt(&self) -> DateTime<Utc> {
        self.created_dt
    }
    pub fn get_expired_dt(&self) -> DateTime<Utc> {
        self.expired_dt
    }
    pub fn default_expires() -> Duration {
        Duration::hours(1)
    }
    pub async fn lookup_by_pk(db: &SqlitePool, pk: i64) -> Result<Self, RustersError> {
        let now = Utc::now();
        query_as::<_, Token>("
            select
                pk,
                hash,
                created_dt,
                expired_dt
            from Tokens
            where pk = $1
            and expired_dt > $2"
        ).bind(pk)
            .bind(now)
            .fetch_one(db)
            .await.quick_match()
    }
    pub async fn insert(
        db: &SqlitePool, hash: impl Hash, expires: Option<Duration>
    ) -> Result<Self, RustersError> {
        let now = Utc::now();
        let exp_dur = match expires {
            Some(d) => d,
            None => Self::default_expires(),
        };
        let exp = Utc::now() + exp_dur;
        let pk = query("
            insert into Tokens (
                hash,
                created_dt,
                expired_dt
            ) values (
                $1,
                $2,
                $3
            );"
        ).bind(hash.get_hash())
            .bind(now)
            .bind(exp)
            .execute(db)
            .await.quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
    pub async fn basic(
        db: &SqlitePool, expires: Option<Duration>
    ) -> Result<Self, RustersError> {
        let hash = Basic::rand()?;
        Self::insert(db, hash, expires).await
    }
    pub async fn secure(
        db: &SqlitePool, expires: Option<Duration>
    ) -> Result<Self, RustersError> {
        let hash = Secure::rand()?;
        Self::insert(db, hash, expires).await
    }
    pub async fn possible<'a>(
        db: &SqlitePool, hash: &'a str
    ) -> Result<Option<Self>, RustersError> {
        let tokens = query_as::<_, Token>("
            select
                pk,
                hash,
                created_dt,
                expired_dt
            from Tokens
            where hash = $1
            and expired_dt > $2"
        ).bind(hash)
            .bind(Utc::now())
            .fetch_all(db)
            .await
            .quick_match()?;
        Ok(tokens.into_iter().nth(0))
    }
    pub async fn lookup<'a>(
        db: &SqlitePool, hash: &'a str
    ) -> Result<Self, RustersError> {
        query_as::<_, Token>("
            select
                pk,
                hash,
                created_dt,
                expired_dt
            from Tokens
            where hash = $1
            and expired_dt > $2"
        ).bind(hash)
            .bind(Utc::now())
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn refresh(
        &mut self, db: &SqlitePool, now_plus: Option<Duration>
    ) -> Result<(), RustersError> {
        let exp_dur = match now_plus {
            Some(d) => d,
            None => Self::default_expires(),
        };
        query("
            update Tokens
            set expired_dt = $1
            where pk = $2"
        ).bind(Utc::now() + exp_dur)
            .bind(self.get_pk())
            .execute(db)
            .await
            .quick_match()?;
        Ok(())
    }
    /// Forces a token to expire
    pub async fn expire(self, db: &SqlitePool) -> Result<(), RustersError> {
        query("
            update Tokens
            set expired_dt = $1
            where pk = $2"
        ).bind(Utc::now())
            .bind(self.get_pk())
            .execute(db)
            .await
            .quick_match()?;
        Ok(())
    }
}
