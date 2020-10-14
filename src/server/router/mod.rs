pub mod get;
pub mod post;
pub mod delete;

pub(crate) use super::Kernel;
use anyhow::Result;
use hyper::{
    Response,
    Body
};

pub fn missing() -> Result<Response<Body>> {
    let res = Response::builder()
        .status(404)
        .body(Body::empty())?;
    Ok(res)
}