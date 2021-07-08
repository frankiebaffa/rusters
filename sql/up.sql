create table if not exists users
	(
		pk integer primary key autoincrement,
		username text not null unique,
		password_hash text not null,
		clearance number check (clearance in (0, 1, 2, 3, 4, 5)),
		created_dt text not null default (datetime(current_timestamp))
	);

