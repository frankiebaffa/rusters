select count(name)
from RustersDb.sqlite_master
where name = 'UsersUniqueUsername'
and type = 'index';
