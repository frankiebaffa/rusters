create table RustersDb.CreateUserTokens
	(
		PK integer not null primary key autoincrement
	,	Token_PK integer not null
	,	Clearance_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	,	foreign key (Clearance_PK) references Clearances (PK)
	);
insert into RustersDb.CreateUserTokens
	(
		Token_PK
	,	Clearance_PK
	,	Created_DT
	)
select
	Token_PK
,	5
,	Created_DT
from RustersDb.ConsumableTokens;
drop table RustersDb.ConsumableTokens;
delete
from RustersDb.Consumers
where Name = 'create_user';
