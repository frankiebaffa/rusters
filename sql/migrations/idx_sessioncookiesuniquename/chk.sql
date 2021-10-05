select count(name)
from RustersDb.sqlite_master
where name = 'SessionCookiesUniqueName'
and type = 'index';
