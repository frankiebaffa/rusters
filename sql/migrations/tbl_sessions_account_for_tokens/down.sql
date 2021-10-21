-- previous version of live table can be completely recreated using tokens table
-- drop live table
drop table RustersDb.Sessions;
-- create new live table
create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement
	,	Hash text not null unique
	,	Created_DT text not null default (datetime('now', 'utc'))
	,	Expired_DT text not null default (datetime('now', 'utc', '+1 hours'))
	);
-- insert hashes from token table
insert into RustersDb.Sessions
	(
		Hash
	,	Created_DT
	,	Expired_DT
	)
select tk.Hash
,	tk.Created_DT
,	tk.Expired_DT
from RustersDb.Tokens as tk
join RustersDb.TokenTypes as tt
on tk.TokenType_PK = tt.PK
and tt.Name = 'Session';
