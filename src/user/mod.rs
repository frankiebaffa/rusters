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
        hash::{
            Secure,
            Hash,
        },
    },
    sqlx::{ FromRow, SqlitePool, query, query_as, },
};
#[derive(FromRow)]
pub struct User {
    pk: i64,
    username: String,
    password_hash: String,
    salt: String,
    active: bool,
    created_dt: DateTime<Utc>,
}
impl User {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_password_hash(&self) -> String {
        self.password_hash.clone()
    }
    pub fn get_salt(&self) -> String {
        self.salt.clone()
    }
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
    pub fn get_active(&self) -> bool {
        self.active
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
                Username,
                PasswordHash,
                Salt,
                Active,
                Created_DT
            from Users
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
                Username,
                PasswordHash,
                Salt,
                Active,
                Created_DT
            from Users
            where Name = $1"
        ).bind(name)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn insert_new<'a>(
        db: &SqlitePool, username: &'a str, password: &'a str
    ) -> Result<Self, RustersError> {
        let hashed = Secure::from_string(password)?;
        let salt = hashed.get_salt();
        let hash = hashed.get_hash();
        let pk = query("
            insert into Users (
                Username,
                PasswordHash,
                Salt,
                Active,
                Created_DT
            ) values (
                $1,
                $2,
                $3,
                $4,
                $5
            )"
        ).bind(username)
            .bind(hash)
            .bind(salt)
            .bind(1_i64)
            .bind(Utc::now())
            .execute(db)
            .await
            .quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
    pub async fn lookup_by_credentials<'a>(
        db: &SqlitePool, username: &'a str, password: &'a str
    ) -> Result<Self, RustersError> {
        let user = match Self::lookup_by_name(db, username).await {
            Ok(user) => user,
            Err(_) => return Err(RustersError::InvalidCredentialsError),
        };
        if Secure::validate(password, &user.password_hash)? {
            Ok(user)
        } else {
            Err(RustersError::InvalidCredentialsError)
        }
    }
}
