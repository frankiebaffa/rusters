use {
    crate::{
        MatchRustersError,
        RustersError,
    },
    sqlx::{ SqlitePool, query_as, query, },
};
pub struct RustersMigrator;
impl RustersMigrator {
    pub async fn migrate(db: &SqlitePool) -> Result<(), RustersError> {
        Self::tbl_users(db).await?;
        Self::tbl_tokens(db).await?;
        Self::tbl_sessions(db).await?;
        Self::tbl_sessioncookies(db).await?;
        Self::tbl_consumers(db).await?;
        Self::tbl_consumable_tokens(db).await?;
        Ok(())
    }
    async fn tbl_users(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(name)
            from sqlite_master
            where type = 'table'
            and name = 'Users'
            limit 1;"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table Users (
                    pk integer not null primary key autoincrement,
                    username text not null unique,
                    password_hash text not null,
                    salt text not null,
                    is_active integer not null default 1,
                    created_dt text not null
                );
                create unique index UsersUniqueUsername
                on Users (username)
                where is_active = 1;"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_tokens(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(name)
            from sqlite_master
            where Name = 'Tokens'
            and Type = 'table';"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table Tokens (
                    pk integer not null primary key autoincrement,
                    hash text not null unique,
                    created_dt not null,
                    expired_dt not null
                );"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_sessions(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(name)
            from sqlite_master
            where type = 'table'
            and name = 'Sessions';"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table Sessions (
                        pk integer not null primary key autoincrement,
                        token_pk integer not null,
                        created_dt text not null,
                        foreign key (token_pk) references Tokens (pk)
                    );"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_sessioncookies(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(name)
            from sqlite_master
            where type = 'table'
            and name = 'SessionCookies'
            limit 1;"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table SessionCookies (
                    pk integer not null primary key autoincrement,
                    session_pk integer not null,
                    name text not null,
                    is_active integer not null default 1,
                    value text not null,
                    created_dt text not null,
                    foreign key (session_pk) references Sessions (pk)
                );
                create unique index SessionCookiesUniqueName
                on SessionCookies (
                    session_pk,
                    name
                )
                where is_active = 1;"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_consumers(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(*)
            from sqlite_master
            where Name = 'Consumers'
            and Type = 'table';"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table Consumers (
                    pk integer primary key autoincrement,
                    name text not null,
                    is_active integer not null default 1,
                    created_dt text not null
                );
                create unique index ConsumersUniqueName
                on Consumers (name)
                where is_active = 1;"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_consumable_tokens(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(*)
            from sqlite_master
            where Name = 'ConsumableTokens'
            and type = 'table';"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table ConsumableTokens (
                    pk integer primary key autoincrement,
                    token_pk integer not null,
                    consumer_pk integer not null,
                    created_dt text not null,
                    foreign key (token_pk) references Tokens (pk),
                    foreign key (consumer_pk) references Consumers (pk)
                );"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
}
