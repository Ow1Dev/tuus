fn main() -> Result<(), Box<dyn std::error::Error>> {
   tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/gateway/v0/gateway.proto"],
            &["../../proto/gateway/v0"],
        )?;
   Ok(())
}
