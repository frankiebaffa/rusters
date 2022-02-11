create table RustersDb.ConsumableTokens
	(
		PK integer primary key autoincrement
	,	Token_PK integer not null
	,	Consumer_PK integer not null
	,	Created_DT text not null
	,	foreign key (Token_PK) references Tokens (PK)
	,	foreign key (Consumer_PK) references Consumers (PK)
	);
insert into RustersDb.Consumers
	(
		Name
	,	Created_DT
	)
select
	'create_user'
,	Created_DT
from RustersDb.CreateUserTokens
limit 1;
insert into RustersDb.ConsumableTokens
	(
		Token_PK
	,	Consumer_PK
	,	Created_DT
	)
select
	Token_PK
,	1
,	Created_DT
from RustersDb.CreateUserTokens;
drop table RustersDb.CreateUserTokens;
