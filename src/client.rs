pub mod reqwest;

use crate::{
    ContentUrl, EndpointUrl, IndexnowError, KeyfileConfig, SubmissionError, SubmissionSuccess,
};
use std::error::Error;
use std::fmt::Debug;
use tower::ServiceExt as _;

pub struct Client<S> {
    endpoint: EndpointUrl,
    keyfile: KeyfileConfig,

    service: S,
}

impl<S, B> Client<S>
where
    S: tower::Service<
            http::Request<crate::Body>,
            Response = http::Response<B>,
            Error: Error + 'static,
        > + Clone,
    B: http_body::Body,
{
    pub fn new(endpoint: EndpointUrl, keyfile: KeyfileConfig, service: S) -> Self {
        Self {
            endpoint,
            keyfile,
            service,
        }
    }

    pub async fn submit_one(&self, url: &ContentUrl) -> Result<SubmissionSuccess, ClientError> {
        let request = crate::submit_one_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            url,
        )
        .map_err(ClientError::Request)?;

        let response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|e| ClientError::Tower(Box::new(e)))?
            .call(request)
            .await
            .map_err(|e| ClientError::Tower(Box::new(e)))?;

        crate::parse_response(&response).map_err(ClientError::IndexNow)
    }

    pub async fn submit_set(&self, urls: &[ContentUrl]) -> Result<SubmissionSuccess, ClientError> {
        let request = crate::submit_set_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            urls,
        )
        .map_err(ClientError::Request)?;

        let response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|e| ClientError::Tower(Box::new(e)))?
            .call(request)
            .await
            .map_err(|e| ClientError::Tower(Box::new(e)))?;

        crate::parse_response(&response).map_err(ClientError::IndexNow)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("request")]
    Request(#[source] IndexnowError),
    #[error("tower-http client")]
    Tower(#[source] Box<dyn Error>),
    #[error("IndexNow API")]
    IndexNow(#[source] SubmissionError),
}
