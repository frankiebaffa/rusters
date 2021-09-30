select count(name)
from RustersDb.sqlite_master
where name = 'Clearances'
and type = 'table';
