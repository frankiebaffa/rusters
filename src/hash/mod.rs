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
pub struct Hashed {
    pub b64_hash: String,
    pub salt: String,
}
pub struct Hasher;
impl Hasher {
    const COST: u32 = DEFAULT_COST;
    const VERSION: Version = Version::TwoB;
    pub fn hash_password(pw: String) -> Result<Hashed, RustersError> {
        let hash_parts = hash_with_result(pw, Self::COST).quick_match()?;
        let salt = hash_parts.get_salt();
        let hash = hash_parts.format_for_version(Self::VERSION);
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(hash.as_bytes()).quick_match()?;
        let b64 = enc_write.into_inner();
        return Ok(Hashed { b64_hash: b64, salt, });
    }
    pub fn verify<'a>(input: &'a str, stored_b64: &'a str) -> Result<bool, RustersError> {
        let mut cur = Cursor::new(stored_b64.as_bytes());
        let mut dec_read = DecoderReader::new(&mut cur, URL_SAFE);
        let mut stored_hash = String::new();
        dec_read.read_to_string(&mut stored_hash).quick_match()?;
        return verify(input, &stored_hash).quick_match();
    }
    pub fn get_token_hash<'a>() -> Result<String, RustersError> {
        let hash = Uuid::new_v4().to_string();
        let mut enc_write = EncoderStringWriter::new(URL_SAFE);
        enc_write.write_all(hash.as_bytes()).quick_match()?;
        let b64 = enc_write.into_inner();
        return Ok(b64);
    }
}
