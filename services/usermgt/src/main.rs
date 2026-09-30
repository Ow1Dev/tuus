use gateway_sdk::{ Context, Problem };

#[derive(serde::Deserialize)]
struct Request {
    name: String,
}

async fn get_users(req: Request, _: Context) -> Result<String, Problem> {
    Ok(format!("get_users, {}!", req.name))
}

async fn get_user(_: Request, _: Context) -> Result<String, Problem> {
    Ok(format!("get_user yay"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    gateway_sdk::new()
        .add_action("get_users", get_users) 
        .add_action("get_user", get_user) 
        .serve(addr)
        .await?;

    Ok(())
}
