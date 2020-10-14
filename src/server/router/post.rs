use tokio::stream::StreamExt;
use super::{missing, Kernel};
use anyhow::Result;
use hyper::{
    Request,
    Response,
    Body
};

pub async fn handle(req: &mut Request<Body>, kernel: Kernel) -> Result<Response<Body>> {
    match req.uri().path() {
        "/upload" => {
            while let Some(Ok(buf)) = req.body_mut().next().await {
                
            };

            missing()
        },
        _ => missing()
    }
}