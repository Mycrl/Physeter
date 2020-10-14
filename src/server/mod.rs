mod router;

use std::convert::Infallible;
use std::net::SocketAddr;
use anyhow::{Result, anyhow};
use hyper::{
    Server,
    service::{
        make_service_fn, 
        service_fn
    }
};

pub async fn run() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router::handle))
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        return Err(anyhow!(format!("{:?}", e)))
    }

    Ok(())
}
