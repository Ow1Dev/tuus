use std::net::{SocketAddr};
use http_body_util::Full;
use hyper::{Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::gateway::{ActionRequest, gateway_client::GatewayClient};

pub mod gateway {
    tonic::include_proto!("gateway");
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = GatewayClient::connect("http://[::1]:50051").await?;

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let client = client.clone();

        let service = service_fn(move |mut _req| {
            let client = client.clone();
            let request = tonic::Request::new(ActionRequest {
                action: "get_users".to_string(),
                data: "{\"name\": \"ow1\"}".to_string(),
            });

            async move {
                let mut client = client.clone();
                let resp = client.action(request).await;
                let msg = resp.unwrap().into_inner().message;

                Ok::<_, hyper::Error>(Response::new(
                    Full::new(Bytes::from(msg))
                ))
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}
