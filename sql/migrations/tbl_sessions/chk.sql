select count(name)
from RustersDb.sqlite_master
where type = 'table'
and name = 'Sessions';

