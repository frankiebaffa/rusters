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
            select count(Name)
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
                    PK integer not null primary key autoincrement,
                    Username text not null unique,
                    PasswordHash text not null,
                    Salt text not null,
                    Active integer not null default 1,
                    Created_DT text not null
                );
                create unique index UsersUniqueUsername
                on Users (Username)
                where Active = 1;"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_tokens(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(Name)
            from sqlite_master
            where Name = 'Tokens'
            and Type = 'table';"
        ).fetch_one(db)
            .await
            .quick_match()?.0 > 0;
        if !exists {
            query("
                create table Tokens (
                    PK integer not null primary key autoincrement,
                    Hash text not null unique,
                    Created_DT not null,
                    Expired_DT not null
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
                        PK integer not null primary key autoincrement,
                        Token_PK integer not null,
                        Created_DT text not null,
                        foreign key (Token_PK) references Tokens (PK)
                    );"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
    async fn tbl_sessioncookies(db: &SqlitePool) -> Result<(), RustersError> {
        let exists = query_as::<_, (i64,)>("
            select count(Name)
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
                    PK integer not null primary key autoincrement,
                    Session_PK integer not null,
                    Name text not null,
                    Active integer not null default 1,
                    Value text not null,
                    Created_DT text not null,
                    foreign key (Session_PK) references Sessions (PK)
                );
                create unique index SessionCookiesUniqueName
                on SessionCookies (
                    Session_PK,
                    Name
                )
                where Active = 1;"
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
                    PK integer primary key autoincrement,
                    Name text not null,
                    IsActive integer not null default 1,
                    Created_DT text not null
                );
                create unique index ConsumersUniqueName
                on Consumers (Name)
                where IsActive = 1;"
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
                    PK integer primary key autoincrement,
                    Token_PK integer not null,
                    Consumer_PK integer not null,
                    Created_DT text not null,
                    foreign key (Token_PK) references Tokens (PK),
                    foreign key (Consumer_PK) references Consumers (PK)
                );"
            ).execute(db)
                .await
                .quick_match()?;
        }
        Ok(())
    }
}
