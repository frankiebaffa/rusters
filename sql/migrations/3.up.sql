create table salts
	(
		pk number autoincrement not null primary key
	,	user_pk number not null
	,	salt_content text not null
	,	created_dt text not null default (datetime(current_timestamp))
	,	foreign key (user_pk) references users (pk)
	);

