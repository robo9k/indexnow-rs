use crate::client::Client;
use crate::{EndpointUrl, KeyfileConfig};
use pin_project::pin_project;
use std::future::Future;
use std::task::{Context, Poll};
use tower::{Layer, Service};

pub struct ReqwestLayer(());

impl<S> tower::Layer<S> for ReqwestLayer {
    type Service = ReqwestService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ReqwestService(service)
    }
}

#[derive(Clone)]
pub struct ReqwestService<S>(S);

impl<S> tower::Service<http::Request<crate::Body>> for ReqwestService<S>
where
    S: Service<reqwest::Request, Response = reqwest::Response, Error = reqwest::Error>,
{
    type Response = http::Response<reqwest::Body>;
    type Error = S::Error;
    type Future = ReqwestFuture<S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<crate::Body>) -> Self::Future {
        match request.map(reqwest::Body::wrap).try_into() {
            Err(err) => ReqwestFuture::Error(Some(err)),
            Ok(request) => {
                let fut = self.0.call(request);
                ReqwestFuture::Future(fut)
            }
        }
    }
}

#[pin_project(project = ReqwestFutureProj)]
pub enum ReqwestFuture<S>
where
    S: Service<reqwest::Request>,
{
    Future(#[pin] S::Future),
    Error(Option<S::Error>),
}

impl<S> Future for ReqwestFuture<S>
where
    S: Service<reqwest::Request>,
    http::Response<reqwest::Body>: From<S::Response>,
{
    type Output = Result<http::Response<reqwest::Body>, S::Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.project() {
            ReqwestFutureProj::Future(fut) => fut.poll(cx).map_ok(From::from),
            ReqwestFutureProj::Error(error) => {
                let error = error.take().expect("Polled after ready");
                Poll::Ready(Err(error))
            }
        }
    }
}

impl Client<ReqwestService<reqwest::Client>> {
    pub fn from_reqwest(
        endpoint: EndpointUrl,
        keyfile: KeyfileConfig,
        client: reqwest::Client,
    ) -> Self {
        let compat = ReqwestLayer(());
        let service = compat.layer(client);
        Client::new(endpoint, keyfile, service)
    }
}
