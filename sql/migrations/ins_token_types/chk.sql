select count(Name)
from RustersDb.TokenTypes
where Name = 'CreateUser'
or Name = 'Session';
