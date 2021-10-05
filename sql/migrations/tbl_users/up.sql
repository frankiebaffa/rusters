create table RustersDb.Users
	(
		PK integer not null primary key autoincrement,
		Username text not null unique,
		PasswordHash text not null,
		Salt text not null,
		Active integer not null default 1,
		Clearance_PK integer not null,
		Created_DT text not null,
		foreign key (Clearance_PK) references Clearances (PK)
	);
