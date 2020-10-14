use super::missing;
use anyhow::Result;
use hyper::{
    Request,
    Response,
    Body
};

pub fn handle(req: &Request<Body>) -> Result<Response<Body>> {
    match req.uri().path() {
        _ => missing()
    }
}