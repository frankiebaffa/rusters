create table RustersDb.Tokens
	(
		PK integer not null primary key autoincrement
	,	TokenType_PK integer not null
	,	Hash text not null unique
	,	Created_DT not null
	,	Expired_DT not null
	,	foreign key (TokenType_PK) references TokenTypes (PK)
	);
