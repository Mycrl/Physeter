use super::{missing, Kernel};
use anyhow::Result;
use hyper::{
    Request,
    Response,
    Body
};

pub fn handle(req: &Request<Body>, kernel: Kernel) -> Result<Response<Body>> {
    match req.uri().path() {
        _ => missing()
    }
}