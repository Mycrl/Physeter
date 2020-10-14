use tokio::stream::StreamExt;
use super::missing;
use anyhow::Result;
use hyper::{
    Request,
    Response,
    Body
};

pub async fn handle(req: &mut Request<Body>) -> Result<Response<Body>> {
    match req.uri().path() {
        "/upload" => {
            let mut size = 0;
            while let Some(Ok(buf)) = req.body_mut().next().await {
                size += buf.len();
            };

            println!("size: {}", size);
            missing()
        },
        _ => missing()
    }
}