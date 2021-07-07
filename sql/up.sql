create table if not exists users
	(
		pk number autoincrement not null primary key,
		username text not null unique,
		password_hash text not null,
		clearance number check (clearance in (0, 1, 2, 3, 4, 5)),
		created_dt text not null default (datetime(current_timestamp)),
		foreign key (clearance_pk) references clearances (pk)
	);
create table if not exists sessions
	(
		pk number autoincrement not null primary key,
		user_pk number not null,
		hash text not null unique,
		created_dt text not null default (datetime(current_timestamp)),
		expired_dt text not null default (datetime(current_timestamp + 1 hours)),
		foreign key (user_pk) references users (pk)
	);

