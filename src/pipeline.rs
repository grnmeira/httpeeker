use std::sync::Arc;
use hudsucker::{
    hyper::{Body, Request, Response},
    async_trait::async_trait,
    HttpHandler, HttpContext, RequestOrResponse
};

pub enum ProcessorResult {
    Continue(Request<Body>),
    Break(Request<Body>),
}

pub trait Processor {
    fn process(&self, req: Request<Body>) -> ProcessorResult;
}

#[derive(Clone)]
pub struct Pipeline {
    pub processors: Arc<Vec<Box<dyn Processor + Send + Sync>>>,
}

#[async_trait]
impl HttpHandler for Pipeline {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        for p in self.processors.iter() {
            let result = p.process(req);
            req = match result {
                ProcessorResult::Continue(req) => req,
                ProcessorResult::Break(req) => {
                    return req.into();
                }
            }
        }
        req.into()
    }
}