mod router;

use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::net::SocketAddr;
use anyhow::{Result, anyhow};
use hyper::{
    Server,
    Method,
    service::{
        make_service_fn, 
        service_fn
    }
};

pub type Kernel = Arc<Mutex<super::Kernel>>;

pub async fn run(kernel: Kernel) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(|mut req| async {
            match req.method() {
                &Method::GET => router::get::handle(&req, kernel.clone()),
                &Method::POST => router::post::handle(&mut req, kernel.clone()).await,
                _ => router::missing()
            }
        }))
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        return Err(anyhow!(format!("{:?}", e)))
    }

    Ok(())
}
