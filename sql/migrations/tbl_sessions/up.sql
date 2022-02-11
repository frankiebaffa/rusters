create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement
	,	Token_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	);
