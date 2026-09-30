fn main() -> Result<(), Box<dyn std::error::Error>> {
   tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &["../../proto/gateway/v0/gateway.proto"],
            &["../../proto/gateway/v0"],
        )?;
   Ok(())
}
