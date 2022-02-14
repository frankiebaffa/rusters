use {
    base64::{
        write::EncoderStringWriter,
        read::DecoderReader,
        URL_SAFE,
    },
    bcrypt::{
        DEFAULT_COST,
        hash_with_result,
        verify,
        Version,
    },
    crate::error::{
        MatchRustersError,
        RustersError,
    },
    std::io::{
        Cursor,
        Read,
        Write,
    },
    uuid::Uuid,
};
pub trait Hash: Sized {
    fn get_hash(&self) -> String;
    fn from_string(to_hash: impl AsRef<str>) -> Result<Self, RustersError>;
    fn rand() -> Result<Self, RustersError> {
        let uuid = Uuid::new_v4().to_string();
        Self::from_string(uuid)
    }
}
pub struct Basic {
    hash: String,
}
impl Hash for Basic {
    fn get_hash(&self) -> String {
        self.hash.clone()
    }
    fn from_string(to_hash: impl AsRef<str>) -> Result<Self, RustersError> {
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(to_hash.as_ref().as_bytes()).quick_match()?;
        let hash = enc_write.into_inner();
        return Ok(Basic { hash });
    }
}
pub struct Secure {
    hash: String,
    salt: String,
}
impl Secure {
    const COST: u32 = DEFAULT_COST;
    const VERSION: Version = Version::TwoB;
    pub fn validate(
        check: impl AsRef<str>, against: impl AsRef<str>
    ) -> Result<bool, RustersError> {
        let mut cur = Cursor::new(against.as_ref().as_bytes());
        let mut dec_read = DecoderReader::new(&mut cur, URL_SAFE);
        let mut stored_hash = String::new();
        dec_read.read_to_string(&mut stored_hash).quick_match()?;
        return verify(check.as_ref(), &stored_hash).quick_match();
    }
    pub fn get_salt(&self) -> String {
        self.salt.clone()
    }
}
impl Hash for Secure {
    fn get_hash(&self) -> String {
        self.hash.clone()
    }
    fn from_string(to_hash: impl AsRef<str>) -> Result<Self, RustersError> {
        let hash_parts = hash_with_result(to_hash.as_ref(), Self::COST).quick_match()?;
        let salt = hash_parts.get_salt();
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(hash.as_bytes()).quick_match()?;
        let hash = enc_write.into_inner();
        return Ok(Secure { hash, salt, });
    }
}
