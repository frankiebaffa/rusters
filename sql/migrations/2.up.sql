create table sessions
	(
		pk number autoincrement not null primary key,
		user_pk number not null,
		hash text not null unique,
		created_dt text not null default (datetime(current_timestamp)),
		expired_dt text not null default (datetime(current_timestamp + 1 hours)),
		foreign key (user_pk) references users (pk)
	);

