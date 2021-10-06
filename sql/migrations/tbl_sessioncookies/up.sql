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
