use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_protos(dir: &Path, protos: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_protos(&path, protos)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                protos.push(path);
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const PROTOBUFS_DIR: &str = "protobufs";

    let mut protos = Vec::with_capacity(1024); // as of writing, 596 protos
    collect_protos(Path::new(PROTOBUFS_DIR), &mut protos)?;
    // println!("cargo::warning=found {} protos", protos.len());

    println!("cargo::rerun-if-changed={}", PROTOBUFS_DIR);
    for proto in &protos {
        println!("cargo::rerun-if-changed={}", proto.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("file_descriptor_set.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&protos, &[PathBuf::from(PROTOBUFS_DIR)])?;

    prost_protovalidate_build::Builder::new()
        .file_descriptor_set_path(&descriptor_path)?
        .compile()?;

    Ok(())
}
