select count(*)
from RustersDb.sqlite_master
where Name = 'Sessions'
and Type = 'table'
and SQL like '%Token_PK%';
