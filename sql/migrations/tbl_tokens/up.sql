create table RustersDb.Tokens
	(
		PK integer not null primary key autoincrement
	,	Hash text not null unique
	,	Created_DT not null
	,	Expired_DT not null
	);
