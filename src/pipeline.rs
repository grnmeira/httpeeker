use hudsucker::{
    async_trait::async_trait,
    hyper::{Body, Request, Response},
    HttpContext, HttpHandler, RequestOrResponse,
};
use std::sync::Arc;

pub enum ProcessorResult<T> {
    Continue(T),
    Break(T),
}

pub trait Processor<T> {
    fn process(&self, r: T) -> ProcessorResult<T>;
}

pub trait OutboundProcessor: Processor<Request<Body>> {}
pub trait InboundProcessor: Processor<Response<Body>> {}

#[derive(Clone)]
pub struct Pipeline {
    outbound_processors: Arc<Vec<Box<dyn OutboundProcessor + Send + Sync>>>,
    inbound_processors: Arc<Vec<Box<dyn InboundProcessor + Send + Sync>>>,
}

impl Pipeline {
    pub fn new(
        outbound_processors: Vec<Box<dyn OutboundProcessor + Send + Sync>>,
        inbound_processors: Vec<Box<dyn InboundProcessor + Send + Sync>>,
    ) -> Pipeline {
        Pipeline {
            outbound_processors: Arc::new(outbound_processors),
            inbound_processors: Arc::new(inbound_processors),
        }
    }
}

pub struct Logger;

impl Processor<Request<Body>> for Logger {
    fn process(&self, req: Request<Body>) -> ProcessorResult<Request<Body>> {
        println!("{}", req.uri().to_string());
        ProcessorResult::Continue(req)
    }
}

impl OutboundProcessor for Logger {}

pub struct Filter {
    pub filter_string: String,
}

impl Processor<Request<Body>> for Filter {
    fn process(&self, req: Request<Body>) -> ProcessorResult<Request<Body>> {
        if req.uri().to_string().contains(self.filter_string.as_str()) {
            ProcessorResult::Continue(req)
        } else {
            ProcessorResult::Break(req)
        }
    }
}

impl OutboundProcessor for Filter {}

#[async_trait]
impl HttpHandler for Pipeline {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        for p in self.outbound_processors.iter() {
            req = match p.process(req) {
                ProcessorResult::Continue(req) => req,
                ProcessorResult::Break(req) => {
                    return req.into();
                }
            }
        }
        return req.into();
    }

    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        mut res: Response<Body>,
    ) -> Response<Body> {
        res
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn creating_basic_pipeline() {}
}
