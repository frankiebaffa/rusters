select count(name)
from RustersDb.sqlite_master
where name = 'ClearancesUniqueSequence'
and type = 'index';
