pub mod reqwest;

use crate::{
    BuildRequestError, ContentUrl, EndpointUrl, KeyfileConfig, ParseResponseError, SubmissionError,
    SubmissionResult, SubmissionSuccess,
};
use std::error::Error;
use std::fmt::Debug;
use tower::ServiceExt as _;

pub struct Client<S> {
    endpoint: EndpointUrl,
    keyfile: KeyfileConfig,

    service: S,
}

fn map_response_result(
    result: Result<SubmissionResult, ParseResponseError>,
) -> Result<SubmissionSuccess, ClientError> {
    match result {
        Ok(domain) => match domain {
            Ok(success) => Ok(success),
            Err(domain_err) => Err(ClientError::IndexNow(domain_err)),
        },
        Err(technical) => Err(ClientError::ParseResponse(technical)),
    }
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
        let request = crate::build_one_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            url,
        )
        .map_err(ClientError::BuildRequest)?;

        let response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|e| ClientError::HttpService(Box::new(e)))?
            .call(request)
            .await
            .map_err(|e| ClientError::HttpService(Box::new(e)))?;

        map_response_result(crate::parse_response(&response))
    }

    pub async fn submit_set(&self, urls: &[ContentUrl]) -> Result<SubmissionSuccess, ClientError> {
        let request = crate::build_set_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            urls,
        )
        .map_err(ClientError::BuildRequest)?;

        let response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|e| ClientError::HttpService(Box::new(e)))?
            .call(request)
            .await
            .map_err(|e| ClientError::HttpService(Box::new(e)))?;

        map_response_result(crate::parse_response(&response))
    }
}

/// Error for [`Client`]
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("can not build request")]
    BuildRequest(#[source] BuildRequestError),
    #[error("tower-http client")]
    HttpService(#[source] Box<dyn Error>),
    #[error("can not parse response")]
    ParseResponse(#[source] ParseResponseError),
    #[error("unsuccessful IndexNow submission")]
    IndexNow(#[source] SubmissionError),
}
