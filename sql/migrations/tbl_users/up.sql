create table RustersDb.Users
	(
		PK integer not null primary key autoincrement,
		Username text not null unique,
		PasswordHash text not null,
		Clearance integer not null default 5 check(Clearance in (0, 1, 2, 3, 4, 5)),
		Created_DT text not null default (datetime(current_timestamp))
	);

