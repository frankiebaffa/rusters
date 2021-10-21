-- create temp
create temporary table Sessions_BKUP
	(
		PK integer not null
	,	Hash text not null
	,	Created_DT text not null
	,	Expired_DT text not null
	);
-- fill temp with live data
insert into Sessions_BKUP
	(
		PK
	,	Hash
	,	Created_DT
	,	Expired_DT
	)
select *
from RustersDb.Sessions;
-- drop live table
drop table RustersDb.Sessions;
-- create new live table
create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement
	,	Token_PK integer not null
	,	Created_DT text not null default (datetime('now', 'utc'))
	,	foreign key (Token_PK) references Tokens (PK)
	);
-- insert hashes into token table
insert into RustersDb.Tokens
	(
		TokenType_PK
	,	Hash
	,	Created_DT
	,	Expired_DT
	)
select
	(
		select PK
		from RustersDb.TokenTypes
		where Name = 'Session'
		limit 1
	)
,	Hash
,	Created_DT
,	Expired_DT
from Sessions_BKUP;
-- insert remaining info into new sessions table
insert into RustersDb.Sessions
	(
		Token_PK
	,	Created_DT
	)
select
	tk.PK
,	ss.Created_DT
from RustersDb.Tokens as tk
join Sessions_BKUP as ss
on tk.Hash = ss.Hash;
-- drop temporary table
drop table Sessions_BKUP;
