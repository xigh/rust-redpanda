fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to rerun this if the proto file changes
    println!("cargo:rerun-if-changed=proto/chat.proto");

    // Generate Rust code from protobuf using prost
    prost_build::compile_protos(&["proto/chat.proto"], &["proto"])?;

    Ok(())
}
