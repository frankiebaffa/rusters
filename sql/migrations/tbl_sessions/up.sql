create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement,
		Hash text not null unique,
		Created_DT text not null default (datetime('now', 'utc')),
		Expired_DT text not null default (datetime('now', 'utc', '+1 hours'))
	);
