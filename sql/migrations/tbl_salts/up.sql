create table RustersDb.Salts
	(
		PK integer primary key autoincrement not null
	,	User_PK integer not null
	,	SaltContent text not null
	,	Created_DT text not null default (datetime(current_timestamp))
	,	foreign key (User_PK) references Users (PK)
	);

