#[derive(Debug, thiserror::Error)]
pub enum IndexnowError {
    #[error("Invalid key")]
    InvalidKey,

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
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
    endpoint: http::Uri,
    key: crate::Key,
    key_location: Option<http::Uri>,
    urls: Vec<http::Uri>,
) -> Result<()> {
    if urls.len() == 1 {
        let request = submit_one_request(endpoint, key, key_location, urls[0].clone()).unwrap();
        println!("Request: {:?}", request);
    }
    todo!();
}

fn submit_one_request(
    endpoint: http::Uri,
    key: crate::Key,
    key_location: Option<http::Uri>,
    url: http::Uri,
) -> Result<http::Request<()>> {
    let mut query = vec![("url", url.to_string()), ("key", key.0)];

    if let Some(key_location) = key_location {
        query.push(("keyLocation", key_location.to_string()));
    }

    let mut path_and_query = endpoint.path().to_owned();
    path_and_query.push('?');
    path_and_query.push_str(
        &serde_urlencoded::to_string(query)
            .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?,
    );

    let mut parts = endpoint.clone().into_parts();
    parts.path_and_query = Some(
        path_and_query
            .parse()
            .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?,
    );

    let request = http::Request::builder()
        .uri(http::Uri::from_parts(parts).map_err(|e| crate::IndexnowError::Other(Box::new(e)))?)
        .method(http::Method::GET);

    Ok(request
        .body(())
        .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_one_request() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let request = submit_one_request(
            DEFAULT_ENDPOINT.clone(),
            "687a308e4eff49f994d89eb22f764514".parse()?,
            None,
            "https://www.example.com/product.html".parse()?,
        )?;

        assert_eq!(*request.uri(), "https://api.indexnow.org/indexnow?url=https%3A%2F%2Fwww.example.com%2Fproduct.html&key=687a308e4eff49f994d89eb22f764514");
        assert_eq!(request.method(), http::Method::GET);

        Ok(())
    }

    #[test]
    fn test_submit_one_request_with_location() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let request = submit_one_request(
            DEFAULT_ENDPOINT.clone(),
            "687a308e4eff49f994d89eb22f764514".parse()?,
            Some("http://www.example.com/myIndexNowKey63638.txt".parse()?),
            "http://www.example.com/product.html".parse()?,
        )?;

        assert_eq!(*request.uri(), "https://api.indexnow.org/indexnow?url=http%3A%2F%2Fwww.example.com%2Fproduct.html&key=687a308e4eff49f994d89eb22f764514&keyLocation=http%3A%2F%2Fwww.example.com%2FmyIndexNowKey63638.txt");
        assert_eq!(request.method(), http::Method::GET);

        Ok(())
    }
}
