select count(name)
from RustersDb.sqlite_master
where name = 'ClearancesUniqueName'
and type = 'index';
