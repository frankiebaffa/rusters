create table RustersDb.TokenTypes
	(
		PK integer not null primary key autoincrement
	,	Name text not null unique
	,	Description text not null unique
	,	Created_DT text not null default (datetime('now', 'utc'))
	);
