create table RustersDb.Tokens
	(
		PK integer not null primary key autoincrement
	,	TokenType_PK integer not null
	,	Hash text not null unique
	,	Created_DT not null default (datetime('now', 'utc'))
	,	Expired_DT not null default (datetime('now', 'utc', '+1 hours'))
	,	foreign key (TokenType_PK) references TokenTypes (PK)
	);
