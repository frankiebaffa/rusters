select count(Name)
from RustersDb.sqlite_master
where Type = 'table'
and Name = 'CreateUserTokens';
