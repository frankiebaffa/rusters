select count(*)
from RustersDb.sqlite_master
where Name = 'ConsumableTokens'
and type = 'table';
