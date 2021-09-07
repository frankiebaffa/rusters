select rowid
,   pk
,   username
,   password_hash
,   clearance_pk
,   created_dt
from users
where rowid = :rowid

