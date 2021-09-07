create table users
	(
		pk number autoincrement not null primary key,
		username text not null unique,
		password_hash text not null,
		clearance number check (clearance in (0, 1, 2, 3, 4, 5)),
		created_dt text not null default (datetime(current_timestamp)),
		foreign key (clearance_pk) references clearances (pk)
	);

