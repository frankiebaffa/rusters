select count(Name)
from RustersDb.sqlite_master
where Name = 'Tokens'
and Type = 'table';
