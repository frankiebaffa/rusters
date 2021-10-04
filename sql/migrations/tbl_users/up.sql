create table RustersDb.Users
	(
		PK integer not null primary key autoincrement,
		Username text not null unique,
		PasswordHash text not null,
		Active integer not null default 1,
		Clearance_PK integer not null,
		Created_DT text not null default (datetime(current_timestamp)),
		foreign key (Clearance_PK) references Clearances (PK)
	);
