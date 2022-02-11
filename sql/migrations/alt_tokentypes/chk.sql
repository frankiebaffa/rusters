select 
	case
		when count(*) > 0
		then 0
		else 1
		end
from RustersDb.sqlite_master
where Name = 'TokenTypes'
and Type = 'table';
