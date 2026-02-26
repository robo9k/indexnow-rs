pub mod client;
pub use client::Client;

#[derive(Debug, thiserror::Error)]
pub enum IndexnowError {
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = std::result::Result<T, crate::IndexnowError>;

// TODO: copy impl instead of type alias
pub type Body = http_body_util::Full<bytes::Bytes>;

/// Absolute URL of a search engine's IndexNow API endpoint
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use indexnow::EndpointUrl;
/// assert_eq!("https://api.indexnow.org/indexnow".parse::<EndpointUrl>()?, EndpointUrl::default());
/// # Ok(())
/// # }
/// ```
// TODO: Clone, PartialEq ↔ &str, TryFrom &[u8], TryFrom Vec<u8>, TryFrom &str, TryFrom String, Hash
// TODO: serde::Serialize, serde::Deserialize try_from
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointUrl(http::Uri);

impl std::fmt::Display for EndpointUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for EndpointUrl {
    type Err = ParseEndpointUrlError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use http::uri::Scheme;

        let uri = s
            .parse::<http::Uri>()
            .map_err(|_e| ParseEndpointUrlError(()))?;

        if let Some(scheme) = uri.scheme() {
            if !(scheme == &Scheme::HTTP || scheme == &Scheme::HTTPS) {
                return Err(ParseEndpointUrlError(()));
            }
        } else {
            return Err(ParseEndpointUrlError(()));
        }

        if let Some(_query) = uri.query() {
            return Err(ParseEndpointUrlError(()));
        }

        let endpoint = Self(uri);
        Ok(endpoint)
    }
}

impl std::default::Default for EndpointUrl {
    fn default() -> Self {
        "https://api.indexnow.org/indexnow"
            .parse()
            .expect("known valid endpoint URL can be parsed")
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid endpoint URL")]
pub struct ParseEndpointUrlError(());

#[derive(Debug, Clone, serde::Serialize)]
pub struct Key(String);

static KEY_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new("^[a-zA-Z0-9\\-]{8,128}$").expect("static regex to be parseable")
});

impl std::str::FromStr for Key {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if KEY_REGEX.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(ParseKeyError(()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid key")]
pub struct ParseKeyError(());

/// Absolute URL of a key file within your host
///
/// This is for [explicit locations of key files](`KeyfileLocation::Url`).
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use indexnow::KeyfileUrl;
/// let keyfile_url = "http://www.example.com/myIndexNowKey63638.txt".parse::<KeyfileUrl>()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(into = "String")]
pub struct KeyfileUrl(http::Uri);

impl std::fmt::Display for KeyfileUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for KeyfileUrl {
    type Err = ParseKeyfileUrlError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use http::uri::Scheme;

        let uri = s
            .parse::<http::Uri>()
            .map_err(|_e| ParseKeyfileUrlError(()))?;

        if let Some(scheme) = uri.scheme() {
            if !(scheme == &Scheme::HTTP || scheme == &Scheme::HTTPS) {
                return Err(ParseKeyfileUrlError(()));
            }
        } else {
            return Err(ParseKeyfileUrlError(()));
        }

        let keyfile = Self(uri);
        Ok(keyfile)
    }
}

impl std::convert::From<KeyfileUrl> for String {
    fn from(value: KeyfileUrl) -> Self {
        value.to_string()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid key file URL")]
pub struct ParseKeyfileUrlError(());

/// Location of a key file on your host
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
// FIXME: Might be cleaner to impl Serialize as Into Option<KeyfileUrl> (RootDirectory null instead of "RootDirectory"), but still needs skip_serializing_if on outer structs
// NOTE: From Option<KeyfileUrl> would also match how CLI works currently
#[serde(untagged)]
pub enum KeyfileLocation {
    /// Implicit location at root directory of the host/URL
    ///
    /// The format is `https://{host}/{key}.txt`.
    /// For a key of `578ca88e0a3941d2a149ae7a54cefc01` and a host of `www.example.com` this is implied to be `https://www.example.org/578ca88e0a3941d2a149ae7a54cefc01.txt`
    #[default]
    RootDirectory,
    /// Absolute URL within the same host
    ///
    /// The filename does not have to match the key / file contents.
    ///
    /// Note that the directory of the keyfile determines valid URLs to be considered by the search engine;
    /// e.g. a location of `http://example.com/catalog/key12457EDd.txt` only allows URLs starting with `http://example.com/catalog/` but not `http://example.com/help/` for example.
    Url(KeyfileUrl),
}

impl KeyfileLocation {
    fn is_rootdirectory(&self) -> bool {
        Self::RootDirectory == *self
    }
}

impl std::convert::From<KeyfileUrl> for KeyfileLocation {
    fn from(value: KeyfileUrl) -> Self {
        Self::Url(value)
    }
}

/// Combined [key](`Key`) its [file location](`KeyfileLocation`)
#[derive(Debug)]
pub struct KeyfileConfig {
    key: Key,
    location: KeyfileLocation,
}

impl KeyfileConfig {
    pub fn new(key: Key, location: KeyfileLocation) -> Self {
        Self { key, location }
    }
}

/// Absolute URL of changed content on your site
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use indexnow::ContentUrl;
/// let content_url = "https://www.example.com/product.html".parse::<ContentUrl>()?;
/// # Ok(())
/// # }
/// ```
// TODO: Clone, PartialEq ↔ &str, TryFrom &[u8], TryFrom Vec<u8>, TryFrom &str, TryFrom String, Hash
// TODO: serde::Serialize, serde::Deserialize try_from
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(into = "String")]
pub struct ContentUrl(http::Uri);

impl std::fmt::Display for ContentUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ContentUrl {
    type Err = ParseContentUrlError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use http::uri::Scheme;

        let uri = s
            .parse::<http::Uri>()
            .map_err(|_e| ParseContentUrlError(()))?;

        if let Some(scheme) = uri.scheme() {
            if !(scheme == &Scheme::HTTP || scheme == &Scheme::HTTPS) {
                return Err(ParseContentUrlError(()));
            }
        } else {
            return Err(ParseContentUrlError(()));
        }

        let content = Self(uri);
        Ok(content)
    }
}

impl std::convert::From<ContentUrl> for String {
    fn from(value: ContentUrl) -> Self {
        value.to_string()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid content URL")]
pub struct ParseContentUrlError(());

// TODO: 1 <= urls.len <= 10_000
// TODO: urls[0].host == key_location.host
// TODO: urls[0].host == urls[*].host
// TODO: urls[*].path startsWith key_location.path.directory
// TODO: is wrangling newtypes infallible?
pub async fn submit(
    endpoint: EndpointUrl,
    key: crate::Key,
    key_location: KeyfileLocation,
    urls: Vec<ContentUrl>,
) -> Result<()> {
    match urls.len() {
        0 => panic!("TODO: need to error out properly here, or do we? what does it mean to submit no URLs? can it fail?"),
        1 => {
            let request = submit_one_request(endpoint, &key, &key_location, &urls[0])?;
            println!("Request: {:?}", request);
        }
        _ => {
            let request = submit_set_request(endpoint, &key, &key_location, &urls)?;
            println!("Request: {:?}", request);
        }
    };

    // sans-io so far, return http::Request<impl Body> ?
    todo!()
}

pub(crate) fn submit_one_request(
    endpoint: EndpointUrl,
    key: &Key,
    key_location: &KeyfileLocation,
    url: &ContentUrl,
) -> Result<http::Request<Body>> {
    #[derive(Debug, serde::Serialize)]
    struct Query<'a> {
        url: &'a ContentUrl,
        key: &'a Key,
        #[serde(
            rename = "keyLocation",
            default,
            skip_serializing_if = "KeyfileLocation::is_rootdirectory"
        )]
        key_location: &'a KeyfileLocation,
    }

    let query = Query {
        url,
        key,
        key_location,
    };

    let mut path_and_query = endpoint.0.path().to_owned();
    path_and_query.push('?');
    path_and_query.push_str(
        &serde_urlencoded::to_string(&query)
            .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?,
    );

    let mut parts = endpoint.0.clone().into_parts();
    parts.path_and_query = Some(
        path_and_query
            .parse()
            .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?,
    );

    let request = http::Request::builder()
        .uri(http::Uri::from_parts(parts).map_err(|e| crate::IndexnowError::Other(Box::new(e)))?)
        .method(http::Method::GET);

    Ok(request
        .body(http_body_util::Full::<bytes::Bytes>::default())
        .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?)
}

pub(crate) fn submit_set_request(
    endpoint: EndpointUrl,
    key: &Key,
    key_location: &KeyfileLocation,
    urls: &[ContentUrl],
) -> Result<http::Request<Body>> {
    #[derive(Debug, serde::Serialize)]
    struct UrlSet<'a> {
        host: &'a str,
        key: &'a Key,
        #[serde(
            rename = "keyLocation",
            default,
            skip_serializing_if = "KeyfileLocation::is_rootdirectory"
        )]
        key_location: &'a KeyfileLocation,
        #[serde(rename = "urlList")]
        url_list: &'a [ContentUrl],
    }

    let host = urls[0]
        .0
        .host()
        .expect("validated newtype URL to have host");

    let request = http::Request::builder()
        .uri(endpoint.0)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            /*headers::ContentType::json()*/ "application/json",
        );

    let url_set = UrlSet {
        host,
        key: &key,
        key_location: &key_location,
        url_list: &urls,
    };

    let body =
        serde_json::to_vec(&url_set).map_err(|e| crate::IndexnowError::Other(Box::new(e)))?;

    Ok(request
        .body(http_body_util::Full::new(bytes::Bytes::from(body)))
        .map_err(|e| crate::IndexnowError::Other(Box::new(e)))?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionSuccess {
    /// Submission was accepted and processed
    Ok,
    /// Submission was accepted, but will be processed later
    ///
    /// Processing might fail key validation.
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAfter {
    Date(std::time::SystemTime),
    Delay(std::time::Duration),
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimitError {
    retry_after: RetryAfter,
}

impl RateLimitError {
    pub fn retry_after(&self) -> RetryAfter {
        self.retry_after
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubmissionError {
    #[error("bad request")]
    BadRequest,
    #[error("forbidden")]
    Forbidden,
    #[error("unprocessable entity")]
    UnprocessableEntity,
    #[error("too many requests")]
    TooManyRequests(RateLimitError),
    // TODO: Other http::Response<B> or just http::Status ?
}

pub fn parse_response<B: http_body::Body>(
    response: &http::Response<B>,
) -> std::result::Result<SubmissionSuccess, SubmissionError> {
    use headers::HeaderMapExt as _;
    use http::StatusCode;

    if response.status() == StatusCode::OK {
        Ok(SubmissionSuccess::Ok)
    } else if response.status() == StatusCode::ACCEPTED {
        Ok(SubmissionSuccess::Accepted)
    } else if response.status() == StatusCode::BAD_REQUEST {
        Err(SubmissionError::BadRequest)
    } else if response.status() == StatusCode::FORBIDDEN {
        Err(SubmissionError::Forbidden)
    } else if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
        Err(SubmissionError::UnprocessableEntity)
    } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = response
            .headers()
            .typed_get()
            .expect("HTTP `Retry-After` response header guaranteed by IndexNow API");
        let retry_after = match retry_after {
            headers_retry_after::RetryAfter::Date(date) => RetryAfter::Date(date),
            headers_retry_after::RetryAfter::Delay(delay) => RetryAfter::Delay(delay),
        };
        let rate_limit = RateLimitError { retry_after };
        Err(SubmissionError::TooManyRequests(rate_limit))
    } else {
        panic!("Unexpected IndexNow API response");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpointurl_display() {
        use std::fmt::Display;
        fn assert_display<T: Display>() {}
        assert_display::<EndpointUrl>();
    }

    #[test]
    fn test_endpointurl_parse() {
        use std::str::FromStr;

        assert!(EndpointUrl::from_str("https://api.indexnow.org/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://indexnow.amazonbot.amazon/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://www.bing.com/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://searchadvisor.naver.com/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://search.seznam.cz/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://yandex.com/indexnow").is_ok());
        assert!(EndpointUrl::from_str("https://indexnow.yep.com/indexnow").is_ok());

        assert!(EndpointUrl::from_str("http://localhost:8080").is_ok());

        assert!(EndpointUrl::from_str(
            "https://api.indexnow.org/indexnow?url=url-changed&key=your-key"
        )
        .is_err());
        assert!(EndpointUrl::from_str("api.indexnow.org").is_err());
    }

    #[test]
    fn test_endpointurl_default() {
        use std::default::Default;
        fn assert_default<T: Default>() {}
        assert_default::<EndpointUrl>();
    }

    #[test]
    fn test_parseendpointerror_debug() {
        use std::fmt::Debug;
        fn assert_debug<T: Debug>() {}
        assert_debug::<ParseEndpointUrlError>();
    }

    #[test]
    fn test_parseendpointerror_error() {
        use std::error::Error;
        fn assert_error<T: Error>() {}
        assert_error::<ParseEndpointUrlError>();
    }

    #[test]
    fn test_submit_one_request() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let request = submit_one_request(
            "https://api.indexnow.org/indexnow".parse()?,
            "687a308e4eff49f994d89eb22f764514".parse()?,
            KeyfileLocation::RootDirectory,
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
            "https://api.indexnow.org/indexnow".parse()?,
            "687a308e4eff49f994d89eb22f764514".parse()?,
            KeyfileLocation::Url("http://www.example.com/myIndexNowKey63638.txt".parse()?),
            "http://www.example.com/product.html".parse()?,
        )?;

        assert_eq!(*request.uri(), "https://api.indexnow.org/indexnow?url=http%3A%2F%2Fwww.example.com%2Fproduct.html&key=687a308e4eff49f994d89eb22f764514&keyLocation=http%3A%2F%2Fwww.example.com%2FmyIndexNowKey63638.txt");
        assert_eq!(request.method(), http::Method::GET);

        Ok(())
    }

    #[tokio::test]
    async fn test_submit_set_request() -> std::result::Result<(), Box<dyn std::error::Error>> {
        use http_body_util::BodyExt as _;

        let request = submit_set_request(
            "https://api.indexnow.org/indexnow".parse()?,
            "687a308e4eff49f994d89eb22f764514".parse()?,
            KeyfileLocation::RootDirectory,
            vec![
                "https://www.example.com/url1".parse()?,
                "https://www.example.com/folder/url2".parse()?,
                "https://www.example.com/url3".parse()?,
            ],
        )?;

        assert_eq!(*request.uri(), "https://api.indexnow.org/indexnow");
        assert_eq!(request.method(), http::Method::POST);
        assert_eq!(
            request.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = request.into_body();
        let body_data = body.collect().await?.to_bytes();
        let json_body: serde_json::Value = serde_json::from_slice(&body_data).unwrap();
        assert_json_diff::assert_json_include!(
            actual: json_body,
            expected: serde_json::json!({
                "host": "www.example.com",
                "key": "687a308e4eff49f994d89eb22f764514",
                "urlList": vec![
                    "https://www.example.com/url1",
                    "https://www.example.com/folder/url2",
                    "https://www.example.com/url3",
                ],
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_submit_set_request_with_location(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use http_body_util::BodyExt as _;

        let request = submit_set_request(
            "https://api.indexnow.org/indexnow".parse()?,
            "687a308e4eff49f994d89eb22f764514".parse()?,
            KeyfileLocation::Url("https://www.example.com/myIndexNowKey63638.txt".parse()?),
            vec![
                "https://www.example.com/url1".parse()?,
                "https://www.example.com/folder/url2".parse()?,
                "https://www.example.com/url3".parse()?,
            ],
        )?;

        assert_eq!(*request.uri(), "https://api.indexnow.org/indexnow");
        assert_eq!(request.method(), http::Method::POST);
        assert_eq!(
            request.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = request.into_body();
        let body_data = body.collect().await?.to_bytes();
        let json_body: serde_json::Value = serde_json::from_slice(&body_data).unwrap();
        assert_json_diff::assert_json_include!(
            actual: json_body,
            expected: serde_json::json!({
                "host": "www.example.com",
                "key": "687a308e4eff49f994d89eb22f764514",
                "keyLocation": "https://www.example.com/myIndexNowKey63638.txt",
                "urlList": vec![
                    "https://www.example.com/url1",
                    "https://www.example.com/folder/url2",
                    "https://www.example.com/url3",
                ],
            })
        );

        Ok(())
    }

    #[test]
    fn test_parse_429_response() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let http_response = http::Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", "666")
            .body(String::new())?;

        let response =
            parse_response(&http_response).expect_err("HTTP response should parse as error");

        match response {
            SubmissionError::TooManyRequests(rate_limit) => {
                assert_eq!(
                    rate_limit.retry_after(),
                    RetryAfter::Delay(std::time::Duration::from_secs(666))
                );
            }
            _ => panic!("unexpected response"),
        }

        Ok(())
    }
}
