create table RustersDb.Consumers
	(
		PK integer primary key autoincrement
	,	Name text not null
	,	IsActive integer not null default 1
	,	Created_DT text not null
	);
create unique index RustersDb.ConsumersUniqueName
on Consumers (Name)
where IsActive = 1;
