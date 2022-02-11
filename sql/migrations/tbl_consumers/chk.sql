select count(*)
from RustersDb.sqlite_master
where Name = 'Consumers'
and Type = 'table';
