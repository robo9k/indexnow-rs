pub enum IndexnowError {}

pub type Result<T> = std::result::Result<T, crate::IndexnowError>;

pub static DEFAULT_ENDPOINT: once_cell::sync::Lazy<http::Uri> = once_cell::sync::Lazy::new(|| {
    "https://api.indexnow.org/indexnow"
        .try_into()
        .expect("static URL to be parseable")
});

pub async fn submit(
    _endpoint: http::Uri,
    _key: String,
    _key_location: Option<http::Uri>,
    _urls: Vec<http::Uri>,
) -> Result<()> {
    todo!();
}

#[cfg(test)]
mod tests {}
