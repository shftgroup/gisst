// Vendored from https://github.com/meilisearch/meilisearch-rust/blob/main/src/reqwest.rs on 2026-08-24
// Reason: Need to use latest version of reqwest (0.13) to remove ring dependency, since version 0.12.28 always uses ring if it uses tls

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_io::AsyncRead;
use pin_project_lite::pin_project;
use serde::{Serialize, de::DeserializeOwned};

use meilisearch_sdk::{
    errors::Error,
    request::{HttpClient, Method, parse_response},
};

#[derive(Debug, Clone, Default)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new(api_key: Option<&str>) -> Result<Self, Error> {
        use reqwest::{ClientBuilder, header};

        let builder = ClientBuilder::new();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&qualified_version()).unwrap(),
        );
        if let Some(api_key) = api_key {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
            );
        }

        let builder = builder.default_headers(headers);
        let client = builder.build().map_err(|e| Error::Other(Box::new(e)))?;

        Ok(ReqwestClient { client })
    }
}
#[async_trait::async_trait]
impl HttpClient for ReqwestClient {
    async fn stream_request<
        Query: Serialize + Send + Sync,
        Body: futures_io::AsyncRead + Send + Sync + 'static,
        Output: DeserializeOwned + 'static,
    >(
        &self,
        url: &str,
        method: Method<Query, Body>,
        content_type: &str,
        expected_status_code: u16,
    ) -> Result<Output, Error> {
        use reqwest::header;

        let query = method.query();
        let query = yaup::to_string(query)?;

        let url = if query.is_empty() {
            url.to_string()
        } else {
            format!("{url}{query}")
        };

        let mut request = self.client.request(verb(&method), &url);

        if let Some(body) = method.into_body() {
            // TODO: Currently reqwest doesn't support streaming data in wasm so we need to collect everything in RAM
            {
                let stream = ReaderStream::new(body);
                let body = reqwest::Body::wrap_stream(stream);

                request = request
                    .header(header::CONTENT_TYPE, content_type)
                    .body(body);
            }
        }

        let response = self
            .client
            .execute(request.build().map_err(|e| Error::Other(Box::new(e)))?)
            .await
            .map_err(|e| Error::Other(Box::new(e)))?;
        let status = response.status().as_u16();
        let mut body = response
            .text()
            .await
            .map_err(|e| Error::Other(Box::new(e)))?;

        if body.is_empty() {
            body = "null".to_string();
        }

        parse_response(status, expected_status_code, &body, url.clone())
    }

    fn is_tokio(&self) -> bool {
        true
    }
}

fn verb<Q, B>(method: &Method<Q, B>) -> reqwest::Method {
    match method {
        Method::Get { .. } => reqwest::Method::GET,
        Method::Delete { .. } => reqwest::Method::DELETE,
        Method::Post { .. } => reqwest::Method::POST,
        Method::Put { .. } => reqwest::Method::PUT,
        Method::Patch { .. } => reqwest::Method::PATCH,
    }
}

pub fn qualified_version() -> String {
    const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

    format!("Meilisearch Rust (v{})", VERSION.unwrap_or("unknown"))
}

pin_project! {
    #[derive(Debug)]
    pub struct ReaderStream<R: AsyncRead> {
        #[pin]
        reader: R,
        buf: BytesMut,
        capacity: usize,
    }
}

impl<R: AsyncRead> ReaderStream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: BytesMut::new(),
            // 8KiB of capacity, the default capacity used by `BufReader` in the std
            capacity: 8 * 1024 * 1024,
        }
    }
}

impl<R: AsyncRead> Stream for ReaderStream<R> {
    type Item = std::io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        if this.buf.capacity() == 0 {
            this.buf.resize(*this.capacity, 0);
        }

        match AsyncRead::poll_read(this.reader, cx, this.buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Ready(Ok(0)) => Poll::Ready(None),
            Poll::Ready(Ok(i)) => {
                let chunk = this.buf.split_to(i);
                Poll::Ready(Some(Ok(chunk.freeze())))
            }
        }
    }
}
