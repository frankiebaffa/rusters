create table RustersDb.CreateUserTokens
	(
		PK integer not null primary key autoincrement
	,	Token_PK integer not null
	,	Clearance_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	,	foreign key (Clearance_PK) references Clearances (PK)
	);
