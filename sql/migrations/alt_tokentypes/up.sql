pragma foreign_keys=OFF;
-- rename existing
alter table RustersDb.Tokens rename to TempOldTokens;
-- create new
create table RustersDb.Tokens
	(
		PK integer not null primary key autoincrement
	,	Hash text not null unique
	,	Created_DT not null
	,	Expired_DT not null
	);
-- fill new
insert into RustersDb.Tokens
select
	PK
,	Hash
,	Created_DT
,	Expired_DT
from RustersDb.TempOldTokens;
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
-- drop old
drop table RustersDb.TempOldSessions;
-- drop old tokens table
drop table RustersDb.TempOldTokens;
/**
 * Finally drop TokenTypes
 */
drop table RustersDb.TokenTypes;
pragma foreign_keys=ON;
