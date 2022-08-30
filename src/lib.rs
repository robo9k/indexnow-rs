#[derive(Debug, thiserror::Error)]
pub enum IndexnowError {
    #[error("Invalid key")]
    InvalidKey,
}

pub type Result<T> = std::result::Result<T, crate::IndexnowError>;

#[derive(Debug)]
pub struct Key(String);

static KEY_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new("^[a-zA-Z0-9\\-]{8,128}$").expect("static regex to be parseable")
});

impl std::str::FromStr for Key {
    type Err = IndexnowError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if KEY_REGEX.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(IndexnowError::InvalidKey)
        }
    }
}

pub static DEFAULT_ENDPOINT: once_cell::sync::Lazy<http::Uri> = once_cell::sync::Lazy::new(|| {
    "https://api.indexnow.org/indexnow"
        .try_into()
        .expect("static URL to be parseable")
});

pub async fn submit(
    _endpoint: http::Uri,
    _key: crate::Key,
    _key_location: Option<http::Uri>,
    _urls: Vec<http::Uri>,
) -> Result<()> {
    todo!();
}

#[cfg(test)]
mod tests {}
