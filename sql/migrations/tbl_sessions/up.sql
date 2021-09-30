create table RustersDb.Sessions
	(
		PK integer not null primary key autoincrement,
		User_PK integer not null,
		Hash text not null unique,
		Created_DT text not null default (datetime(current_timestamp)),
		Expired_DT text not null default (datetime(current_timestamp, '+1 hours')),
		foreign key (User_PK) references Users (PK)
	);
