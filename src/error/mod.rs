use {
    bcrypt::BcryptError,
    buildlite::BuildliteError,
    worm::core::sql::Error as RusqliteError,
    std::io::Error as IOError,
};
#[derive(Debug)]
pub enum RustersError {
    BcryptError(BcryptError),
    InvalidCredentialsError,
    IOError(IOError),
    NotLoggedInError,
    SQLError(RusqliteError),
    NoSessionError,
    BuildliteError(BuildliteError),
}
impl std::fmt::Display for RustersError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RustersError::BcryptError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
            RustersError::InvalidCredentialsError => {
                f.write_str("Invalid credentials")
            },
            RustersError::IOError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
            RustersError::NotLoggedInError => {
                f.write_str("Not logged in")
            },
            RustersError::SQLError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
            RustersError::NoSessionError => {
                f.write_str("The session is expired or does not exist")
            },
            RustersError::BuildliteError(e) => {
                let msg = &format!("{}", e);
                f.write_str(msg)
            },
        }
    }
}
impl std::error::Error for RustersError {}
pub trait MatchRustersError<T, U: std::error::Error>: Sized {
    fn quick_match(self) -> Result<T, RustersError>;
}
impl<T> MatchRustersError<T, BcryptError> for Result<T, BcryptError> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::BcryptError(e)),
        };
    }
}
impl<T> MatchRustersError<T, RusqliteError> for Result<T, RusqliteError> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::SQLError(e)),
        };
    }
}
impl<T> MatchRustersError<T, std::io::Error> for Result<T, std::io::Error> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::IOError(e)),
        };
    }
}
impl<T> MatchRustersError<T, BuildliteError> for Result<T, BuildliteError> {
    fn quick_match(self) -> Result<T, RustersError> {
        return match self {
            Ok(s) => Ok(s),
            Err(e) => Err(RustersError::BuildliteError(e)),
        };
    }
}
