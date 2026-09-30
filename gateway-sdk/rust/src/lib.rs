use std::{collections::HashMap, net::SocketAddr, pin::Pin};
use std::sync::Arc;

use tonic::{Request, Response, Status, transport::{Error, Server}};

use crate::gateway::{ActionReply, ActionRequest, gateway_server::{Gateway, GatewayServer}};

mod gateway {
    tonic::include_proto!("gateway");
}

pub struct Problem {
    pub msg: String
}

impl Problem {
    fn bad_request(msg: String) -> Problem {
        Problem { msg }
    }
}

pub struct Context {}

pub struct Service {
    actions: HashMap<String, ActionHandler>,
}

#[tonic::async_trait]
impl Gateway for Service {
    async fn action(&self, request: Request<ActionRequest>,) -> Result<Response<ActionReply>, Status> {
        let r = request.into_inner();
        println!("Getting action {}", r.action);
        let handler = self
            .actions
            .get(&r.action)
            .ok_or_else(|| Status::not_found("action not found"))?;

        let result = handler(r.data.into_bytes(), Context {})
            .await
            .map_err(|e| Status::internal(e.msg))?;

        Ok(Response::new(ActionReply {
            message: result,
        }))
    }
}

type ActionHandler = Box<
    dyn Fn(Vec<u8>, Context)
        -> Pin<Box<dyn Future<Output = Result<String, Problem>> + Send>>
        + Send
        + Sync,
>;

pub fn new() -> Service {
    Service {
        actions: HashMap::new(),
    }
}

impl Service {
    pub fn add_action<R, F, Fut>(
        mut self,
        addr: &str,
        handler: F,
    ) -> Self
    where
        R: serde::de::DeserializeOwned + Send + 'static,
        F: Fn(R, Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, Problem>> + Send + 'static,
    {
        let handler = Arc::new(handler);

        self.actions.insert(
            addr.to_string(),
            Box::new(move |data, ctx| {
                let handler = Arc::clone(&handler);

                Box::pin(async move {
                    let req: R = serde_json::from_slice(&data)
                        .map_err(|e| Problem::bad_request(e.to_string()))?;

                    handler(req, ctx).await
                })
            }),
        );

        self
    }

    pub async fn serve(self, addr: SocketAddr) -> Result<(), Error> {
        Server::builder()
            .add_service(GatewayServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}
