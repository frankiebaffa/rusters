insert into RustersDb.TokenTypes
	(
		Name
	,	Description
	)
select 'CreateUser'
,	'A token generated to allow for user creation'
union all
select 'Session'
,	'A token generated which represents a session';
-- other possibilities could be password reset, etc
