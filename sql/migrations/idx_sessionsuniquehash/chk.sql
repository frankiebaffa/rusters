select count(name)
from RustersDb.sqlite_master
where name = 'SessionsUniqueHash'
and type = 'index';
