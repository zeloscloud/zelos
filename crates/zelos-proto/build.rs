use std::io::Result;

fn main() -> Result<()> {
    let protoc_bin = protoc_bin_vendored::protoc_bin_path().unwrap();

    let mut prost_config = prost_build::Config::new();
    prost_config.protoc_executable(protoc_bin);

    tonic_build::configure().compile_protos_with_config(
        prost_config,
        &[
            "proto/zeloscloud/trace/publish.proto",
            "proto/zeloscloud/trace/subscribe.proto",
            "proto/zeloscloud/trace/trace.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
