use std::net::{SocketAddr};
use http_body_util::Full;
use hyper::{Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tonic::transport::Channel;

use crate::gateway::{ActionRequest, gateway_client::GatewayClient};

pub mod gateway {
    tonic::include_proto!("gateway");
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel = Channel::from_static("http://[::1]:50051")
        .connect_lazy();
    let client = GatewayClient::new(channel);

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let client = client.clone();

        let service = service_fn(move |req| {
            let mut client = client.clone();

            async move {
                let path: Vec<&str> = req.uri().path().split('/').collect();
                if path.iter().count() == 2 {
                    return Ok::<_, hyper::Error>(
                        Response::builder()
                            .status(400)
                            .body(Full::new(
                                Bytes::from("url should be proxy and action")
                            ))
                            .unwrap()
                    )
                }

                let action = path[2];

                let request = tonic::Request::new(ActionRequest {
                    action: action.to_string(),
                    data: "{\"name\": \"ow1\"}".to_string(),
                });
                
                match client.action(request).await {
                   Ok(response) => {
                        let msg = response.into_inner().message;
                        Ok::<_, hyper::Error>(
                            Response::new(Full::new(Bytes::from(msg)))
                        )
                    }
                    Err(status) => {
                        println!("gRPC error: {status}");

                        Ok::<_, hyper::Error>(
                            Response::builder()
                                .status(503)
                                .body(Full::new(
                                    Bytes::from("Service unavailable")
                                ))
                                .unwrap()
                        )
                    }
                }
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}
