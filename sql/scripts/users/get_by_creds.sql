select rowid
,   pk
,   username
,   password_hash
,   clearance_pk
,   created_dt
from users
where username = :username
and password_hash = :password_hash
