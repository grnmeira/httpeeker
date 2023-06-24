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
    outbound_processors: Arc<Vec<Box<dyn Processor + Send + Sync>>>,
    inbound_processors: Arc<Vec<Box<dyn Processor + Send + Sync>>>
}

impl Pipeline {
    pub fn new(
        inbound_processors: Vec<Box<dyn Processor + Send + Sync>>, 
        outbound_processors: Vec<Box<dyn Processor + Send + Sync>>
    ) -> Pipeline {
        Pipeline { 
            inbound_processors: Arc::new(inbound_processors), 
            outbound_processors: Arc::new(outbound_processors)
        }
    }
}

#[async_trait]
impl HttpHandler for Pipeline {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        for p in self.outbound_processors.iter() {
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
    fn creating_basic_pipeline() {
        assert_eq!(true, true);
    }
}