pub mod reqwest;

use crate::{ContentUrl, EndpointUrl, KeyfileConfig};
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
    S: tower::Service<http::Request<crate::Body>, Response = http::Response<B>, Error: Error>
        + Clone,
    B: http_body::Body,
{
    pub fn new(endpoint: EndpointUrl, keyfile: KeyfileConfig, service: S) -> Self {
        Self {
            endpoint,
            keyfile,
            service,
        }
    }

    pub async fn submit_one(&self, url: &ContentUrl) -> Result<(), ClientError> {
        let request = crate::submit_one_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            url,
        )
        .map_err(|_e| ClientError(()))?;

        let _response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|_e| ClientError(()))?
            .call(request)
            .await
            .map_err(|_e| ClientError(()))?;

        todo!()
    }

    pub async fn submit_set(&self, urls: &[ContentUrl]) -> Result<(), ClientError> {
        let request = crate::submit_set_request(
            self.endpoint.clone(),
            &self.keyfile.key,
            &self.keyfile.location,
            urls,
        )
        .map_err(|_e| ClientError(()))?;

        let _response = self
            .service
            .clone()
            .ready()
            .await
            .map_err(|_e| ClientError(()))?
            .call(request)
            .await
            .map_err(|_e| ClientError(()))?;

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Client error")]
pub struct ClientError(());
