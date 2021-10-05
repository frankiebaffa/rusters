select count(Name)
from RustersDb.sqlite_master
where type = 'table'
and name = 'SessionCookies'
limit 1;
