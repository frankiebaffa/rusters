select count(Name)
from RustersDb.sqlite_master
where Name = 'TokenTypes'
and Type = 'table';
