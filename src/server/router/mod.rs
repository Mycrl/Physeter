mod get;
mod post;
mod delete;

use anyhow::Result;
use hyper::{
    Request,
    Response,
    Method,
    Body
};

pub fn missing() -> Result<Response<Body>> {
    let res = Response::builder()
        .status(404)
        .body(Body::empty())?;
    Ok(res)
}

pub async fn handle(mut req: Request<Body>) -> Result<Response<Body>> {
    match req.method() {
        &Method::GET => get::handle(&req),
        &Method::POST => post::handle(&mut req).await,
        _ => missing()
    }
}