pub mod cookie;
use {
    buildlite::{
        BuildliteError,
        Query,
    },
    chrono::{
        DateTime,
        Duration,
        Utc,
    },
    crate::{
        error::{
            MatchRustersError,
            RustersError,
        },
        hash::Hasher,
        token::Token,
        user::User,
    },
    cookie::SessionCookie,
    worm::{
        core::{
            DbCtx,
            PrimaryKey,
            UniqueNameModel,
        },
        derive::Worm,
    },
};
#[derive(Worm)]
#[dbmodel(table(schema="RustersDb", name="Sessions", alias="session"))]
pub struct Session {
    #[dbcolumn(column(name="PK", primary_key))]
    pk: i64,
    #[dbcolumn(column(name="Token_PK", foreign_key="Token", insertable))]
    token_pk: i64,
    #[dbcolumn(column(name="Created_DT", insertable))]
    created_dt: DateTime<Utc>,
}
impl Session {
    pub fn create_new(db: &mut impl DbCtx) -> Result<(Session, String), RustersError> {
        let token = Token::generate_for_new_session(db)?;
        let session = Session::insert_new(db, token.get_id(), Utc::now()).quick_match()?;
        return Ok((session, token.get_hash()));
    }
    pub fn get_active<'a>(db: &mut impl DbCtx, hash: &'a str) -> Result<Session, RustersError> {
        let now: DateTime<Utc> = Utc::now();
        let session = Query::<Session>::select()
            .join_fk::<Token>().join_and()
            .join_fk_gt::<Token>(Token::EXPIRED_DT, &now).join_and()
            .join_fk_eq::<Token>(Token::HASH, &hash)
            .execute_row(db)
            .quick_match()?;
        let exp: DateTime<Utc> = Utc::now() + Duration::hours(1);
        session.update_expired(db, exp)?;
        return Ok(session);
    }
    pub fn get_hash<'a>(&self, db: &mut impl DbCtx) -> Result<String, RustersError> {
        let token = Query::<Token>::select()
            .join::<Self>()
            .where_eq::<Self>(Self::PRIMARY_KEY, &self.get_id())
            .execute_row(db)
            .quick_match()?;
        return Ok(token.get_hash());
    }
    const LOGIN_COOKIE: &'static str = "LOGIN";
    pub fn delete_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str) -> Result<bool, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let aug = Query::<SessionCookie>::update()
            .set(SessionCookie::ACTIVE, &0)
            .where_eq::<SessionCookie>(SessionCookie::SESSION_PK, &self.pk).and()
            .where_eq::<SessionCookie>(SessionCookie::NAME, &name).and()
            .where_eq::<SessionCookie>(SessionCookie::ACTIVE, &1)
            .execute_update(db)
            .quick_match()?;
        if aug > 0 {
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    pub fn read_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str) -> Result<Option<SessionCookie>, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let cookie_res = Query::<SessionCookie>::select()
            .where_eq::<SessionCookie>(SessionCookie::SESSION_PK, &self.get_id()).and()
            .where_eq::<SessionCookie>(SessionCookie::NAME, &name).and()
            .where_eq::<SessionCookie>(SessionCookie::ACTIVE, &1)
            .execute_row(db);
        match cookie_res {
            Ok(c) => return Ok(Some(c)),
            Err(e) => return match e {
                BuildliteError::NoRowsError => Ok(None),
                _ => Err(e).quick_match()?,
            },
        }
    }
    pub fn set_cookie<'a>(&self, db: &mut impl DbCtx, name: &'a str, value: &'a str) -> Result<SessionCookie, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let cookie = self.read_cookie(db, name)?;
        if cookie.is_some() {
            self.delete_cookie(db, name)?;
            let new_cookie = SessionCookie::insert_new(db, self.get_id(), name.to_string(), value.to_string(), Utc::now()).quick_match()?;
            return Ok(new_cookie);
        } else {
            return Ok(SessionCookie::insert_new(
                db, self.get_id(), name.to_string(),
                value.to_string(), Utc::now()
            ).quick_match()?);
        }
    }
    pub fn is_logged_in<'s>(&self, db: &mut impl DbCtx) -> Result<Option<User>, RustersError> {
        self.update_expired(db, Utc::now() + Duration::hours(1))?;
        let login_cookie_opt = self.read_cookie(db, Self::LOGIN_COOKIE)?;
        if login_cookie_opt.is_none() {
            return Ok(None);
        }
        let login_cookie = login_cookie_opt.unwrap();
        let user = User::get_by_name(db, &login_cookie.get_value()).quick_match()?;
        return Ok(Some(user));
    }
    pub fn login<'a>(&self, db: &mut impl DbCtx, username: &'a str, password: &'a str) -> Result<User, RustersError> {
        self.update_expired(db, Utc::now())?;
        let user = User::get_by_name(db, username).quick_match()?;
        let stored_hash = user.get_password_hash();
        let verified = Hasher::verify(password, &stored_hash)?;
        if !verified {
            return Err(RustersError::InvalidCredentialsError);
        }
        self.set_cookie(db, Session::LOGIN_COOKIE, &username)?;
        return Ok(user);
    }
    pub fn log_out<'a>(&self, db: &mut impl DbCtx) -> Result<bool, RustersError> {
        let user_opt = self.is_logged_in(db)?;
        if user_opt.is_some() {
            return Ok(self.delete_cookie(db, Self::LOGIN_COOKIE)?);
        } else {
            return Ok(false);
        }
    }
    fn update_expired(&self, db: &mut impl DbCtx, new_exp: DateTime<Utc>) -> Result<bool, RustersError> {
        let token = Query::<Token>::select()
            .join::<Session>()
            .where_eq::<Session>(Session::PK, &self.pk)
            .execute_row(db)
            .quick_match()?;
        let aug = Query::<Token>::update()
            .set(Token::EXPIRED_DT, &new_exp)
            .where_eq::<Token>(Token::PK, &token.get_id())
            .execute_update(db)
            .quick_match()?;
        if aug == 1 {
            return Ok(true);
        } else if aug > 1 {
            panic!("More than one row updated");
        } else {
            return Ok(false);
        }
    }
}
