use http::Extensions;
use log::{log, Level};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};

pub struct LoggingMiddleware {
    level: Level,
}

impl LoggingMiddleware {
    pub fn new(level: Level) -> Self {
        Self { level }
    }
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        log!(
            self.level,
            "begin request {} {}",
            request.method(),
            request.url()
        );
        let result = next.run(request, extensions).await;
        match result.as_ref() {
            Ok(response) => {
                log!(self.level, "received response {}", response.status());
            }
            Err(e) => {
                log!(self.level, "request failed {:?}", e);
            }
        }
        result
    }
}
