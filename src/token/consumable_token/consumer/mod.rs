use {
    chrono::{
        DateTime,
        Utc,
    },
    crate::error::{
        MatchRustersError,
        RustersError,
    },
    sqlx::{ FromRow, SqlitePool, query, query_as, },
};
#[derive(FromRow)]
pub struct Consumer {
    pk: i64,
    name: String,
    is_active: bool,
    created_dt: DateTime<Utc>,
}
impl Consumer {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_is_active(&self) -> bool {
        self.is_active
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
                Name,
                IsActive,
                Created_DT
            from Consumers
            where PK = $1"
        ).bind(pk)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn lookup_by_name<'a>(
        db: &SqlitePool, name: &'a str
    ) -> Result<Self, RustersError> {
        query_as::<_, Self>("
            select
                PK,
                Name,
                IsActive,
                Created_DT
            from Consumers
            where Name = $1"
        ).bind(name)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn insert_new<'a>(
        db: &SqlitePool, name: &'a str
    ) -> Result<Self, RustersError> {
        let pk = query("
            insert into Consumers (
                Name,
                IsActive,
                Created_DT
            ) values (
                $1,
                $2,
                $3
            )"
        ).bind(name)
            .bind(1_i64)
            .bind(Utc::now())
            .execute(db)
            .await
            .quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
    pub async fn get_or_create(
        db: &SqlitePool, name: impl AsRef<str>
    ) -> Result<Self, RustersError> {
        let n = name.as_ref();
        match Self::lookup_by_name(db, n).await {
            Ok(c) => Ok(c),
            Err(_) => Self::insert_new(db, n).await
        }
    }
}
