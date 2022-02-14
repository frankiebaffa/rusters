pragma foreign_keys=OFF;
-- create new
create table RustersDb.TokenTypes
	(
		PK integer not null primary key autoincrement
	,	Name text not null unique
	,	Description text not null unique
	,	Created_DT text not null default (datetime('now', 'utc'))
	);
-- populate new
insert into RustersDb.TokenTypes
	(
		Name
	,	Description
	)
select 'CreateUser'
,	'A token generated to allow for user creation'
union all
select 'Session'
,	'A token generated which represents a session';
-- rename existing
alter table RustersDb.Tokens rename to TempOldTokens;
-- create new
create table RustersDb.Tokens
	(
		PK integer not null primary key autoincrement
	,	TokenType_PK integer not null
	,	Hash text not null unique
	,	Created_DT not null
	,	Expired_DT not null
	,	foreign key (TokenType_PK) references TokenTypes (PK)
	);
-- fill new with sessions
insert into RustersDb.Tokens
select
	oldtoken.PK
,	(
		select PK
		from RustersDb.TokenTypes
		where Name = 'Session'
		limit 1
	)
,	oldtoken.Hash
,	oldtoken.Created_DT
,	oldtoken.Expired_DT
from RustersDb.TempOldTokens as oldtoken
join RustersDb.Sessions as session
on oldtoken.PK = session.Token_PK;
-- fill new with createuser
insert into RustersDb.Tokens
select
	oldtoken.PK
,	(
		select PK
		from RustersDb.TokenTypes
		where Name = 'CreateUser'
		limit 1
	)
,	oldtoken.Hash
,	oldtoken.Created_DT
,	oldtoken.Expired_DT
from RustersDb.TempOldTokens as oldtoken
join RustersDb.ConsumableTokens as consumable
on oldtoken.PK = consumable.Token_PK;
/**
 * Fix table ConsumableTokens
 */
-- rename existing
alter table RustersDb.ConsumableTokens rename to TempOldConsumableTokens;
-- create new
create table RustersDb.ConsumableTokens
	(
		PK integer primary key autoincrement
	,	Token_PK integer not null
	,	Consumer_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	,	foreign key (Consumer_PK) references Consumers (PK)
	);
-- fill new
insert into RustersDb.ConsumableTokens
select *
from RustersDb.TempOldConsumableTokens;
-- drop old
drop table RustersDb.TempOldConsumableTokens;
/**
 * Fix table Sessions
 */
-- rename old
alter table RustersDb.Sessions rename to TempOldSessions;
-- create new
create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement
	,	Token_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	);
-- fill new
insert into RustersDb.Sessions
select *
from TempOldSessions;
/**
 * Fix table SessionCookies
 */
-- drop unique index
drop index RustersDb.SessionCookiesUniqueName;
-- rename old
alter table RustersDb.SessionCookies rename to TempOldCookies;
-- create new
create table RustersDb.SessionCookies
	(
		PK integer not null primary key autoincrement
	,	Session_PK integer not null
	,	Name text not null
	,	Active integer not null default 1
	,	Value text not null
	,	Created_DT text not null
	,	foreign key (Session_PK) references Sessions (PK)
	);
-- recreate index
create unique index RustersDb.SessionCookiesUniqueName on SessionCookies (Session_PK, Name) where Active = 1;
-- fill table
insert into RustersDb.SessionCookies
select *
from RustersDb.TempOldCookies;
-- drop old cookies table
drop table RustersDb.TempOldCookies;
-- drop old
drop table RustersDb.TempOldSessions;
-- drop old tokens table
drop table RustersDb.TempOldTokens;
