use axum::{
    http::Request,
    response::{IntoResponse, Response},
};
use tower_http::services::ServeDir;

pub struct SelectiveServeDir {
    inner_compressed: ServeDir,
    inner_uncompressed: ServeDir,
    ic_ready: bool,
    iu_ready: bool,
}
impl Clone for SelectiveServeDir {
    fn clone(&self) -> Self {
        Self {
            inner_compressed: self.inner_compressed.clone(),
            inner_uncompressed: self.inner_uncompressed.clone(),
            // the cloned services probably wont be ready
            ic_ready: false,
            iu_ready: false,
        }
    }
}
impl SelectiveServeDir {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            inner_compressed: ServeDir::new(&path).precompressed_gzip(),
            inner_uncompressed: ServeDir::new(path),
            ic_ready: false,
            iu_ready: false,
        }
    }
}

// borrowed from https://github.com/tokio-rs/axum/blob/main/examples/rest-grpc-multiplex/src/multiplex_service.rs

impl tower::Service<Request<axum::body::Body>> for SelectiveServeDir {
    type Response = Response;
    type Error = std::convert::Infallible;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        loop {
            match (self.ic_ready, self.iu_ready) {
                (true, true) => {
                    return Ok(()).into();
                }
                (false, _) => {
                    futures::ready!(
                        <ServeDir as tower::Service<Request<axum::body::Body>>>::poll_ready(
                            &mut self.inner_compressed,
                            cx
                        )
                    )
                    .map_err(|err| match err {})
                    .unwrap();
                    self.ic_ready = true;
                }
                (_, false) => {
                    futures::ready!(
                        <ServeDir as tower::Service<Request<axum::body::Body>>>::poll_ready(
                            &mut self.inner_uncompressed,
                            cx
                        )
                    )
                    .unwrap();
                    self.iu_ready = true;
                }
            }
        }
    }
    fn call(&mut self, req: Request<axum::body::Body>) -> Self::Future {
        assert!(
            self.ic_ready,
            "Compressed ServeDir service not ready. Did you forget to call `poll_ready`?"
        );
        assert!(
            self.iu_ready,
            "Uncompressed ServeDir service not ready. Did you forget to call `poll_ready`?"
        );
        if is_uncompressed_request(&req) {
            self.iu_ready = false;
            let future = self.inner_uncompressed.call(req);
            Box::pin(async move {
                let res = future.await?;
                let mut resp = res.into_response();
                add_headers(&mut resp);
                Ok(resp)
            })
        } else {
            self.ic_ready = false;
            let future = self.inner_compressed.call(req);
            Box::pin(async move {
                let res = future.await?;
                let mut resp = res.into_response();
                add_headers(&mut resp);
                Ok(resp)
            })
        }
    }
}

fn add_headers(resp: &mut Response) {
    let headers = resp.headers_mut();
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        "require-corp".parse().unwrap(),
    );
    headers.insert(
        "Cross-Origin-Resource-Policy",
        "cross-origin".parse().unwrap(),
    );
    headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
}

fn is_uncompressed_request<B>(req: &Request<B>) -> bool {
    req.headers()
        .get("X-Accept-Encoding")
        .map(|xae| xae == "identity")
        .unwrap_or(false)
}
