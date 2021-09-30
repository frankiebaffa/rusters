select count(Name)
from RustersDb.sqlite_master
where type = 'table'
and name = 'Salts';

