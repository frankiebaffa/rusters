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
        session::Session,
        user::User,
    },
    sqlx::{ FromRow, SqlitePool, query, query_as, },
};
#[derive(FromRow)]
pub struct SessionCookie {
    pk: i64,
    session_pk: i64,
    name: String,
    value: String,
    is_active: bool,
    created_dt: DateTime<Utc>,
}
impl SessionCookie {
    pub fn get_pk(&self) -> i64 {
        self.pk
    }
    pub fn get_session_pk(&self) -> i64 {
        self.session_pk
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_value(&self) -> String {
        self.value.clone()
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
                pk,
                session_pk,
                name,
                value,
                is_active,
                created_dt
            from SessionCookies
            where pk = $1
            and is_active = 1;"
        ).bind(pk)
            .fetch_one(db)
            .await
            .quick_match()
    }
    pub async fn create<'a>(
        db: &SqlitePool, session: &Session, name: &'a str, value: &'a str
    ) -> Result<Self, RustersError> {
        let pk = query("
            insert into SessionCookies (
                session_pk,
                name,
                value,
                is_active,
                created_dt
            ) values (
                $1,
                $2,
                $3,
                $4,
                $5
            )"
        ).bind(session.get_pk())
            .bind(name)
            .bind(value)
            .bind(1)
            .bind(Utc::now())
            .execute(db)
            .await
            .quick_match()?
            .last_insert_rowid();
        Self::lookup_by_pk(db, pk).await
    }
    pub async fn delete<'a>(
        db: &SqlitePool, session: &Session, name: &'a str
    ) -> Result<(), RustersError> {
        query("
            update SessionCookies
            set is_active = 0
            where session_pk = $1
            and name = $2
            and is_active = 1"
        ).bind(session.get_pk())
            .bind(name)
            .execute(db)
            .await
            .quick_match()?;
        Ok(())
    }
    pub async fn read<'a>(
        db: &SqlitePool, session: &Session, name: &'a str
    ) -> Result<Option<Self>, RustersError> {
        let cookies = query_as::<_, Self>("
            select
                pk,
                session_pk,
                name,
                value,
                is_active,
                created_dt
            from SessionCookies
            where session_pk = $1
            and name = $2
            and is_active = 1"
        ).bind(session.get_pk())
            .bind(name)
            .fetch_all(db)
            .await
            .quick_match()?;
        Ok(cookies.into_iter().nth(0))
    }
    pub async fn set<'a>(
        db: &SqlitePool, session: &Session, name: &'a str, value: &'a str
    ) -> Result<Self, RustersError> {
        let existing = Self::read(db, session, name).await?;
        match existing {
            Some(cookie) => {
                if cookie.value.eq(value) {
                    Ok(cookie)
                } else {
                    Self::delete(db, session, name).await?;
                    Self::create(db, session, name, value).await
                }
            },
            None => {
                Self::create(db, session, name, value).await
            },
        }
    }
    pub const LOGIN_COOKIE: &'static str = "LOGIN";
    pub async fn has_login_cookie(
        db: &SqlitePool, session: &Session
    ) -> Result<bool, RustersError> {
        Ok(Self::read(db, session, Self::LOGIN_COOKIE).await?.is_some())
    }
    pub async fn login(
        db: &SqlitePool, session: &Session, user: &User
    ) -> Result<Self, RustersError> {
        Self::set(db, session, Self::LOGIN_COOKIE, &user.get_username()).await
    }
    pub async fn logout(
        db: &SqlitePool, session: &Session
    ) -> Result<(), RustersError> {
        Self::delete(db, session, Self::LOGIN_COOKIE).await
    }
}
