use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = args.first().map_or("", String::as_str);

    match command {
        "generate-header" => generate_header(args.iter().any(|arg| arg == "--check")),
        _ => Err("usage: cargo run -p xtask -- generate-header [--check]".into()),
    }
}

fn generate_header(check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo_root()?;
    let output_path =
        repo_root.join("apple/macos/Sources/AshenFrontierBridge/include/ashen_frontier.h");
    let generated = generate_c_header(&repo_root)?;

    if check {
        let current = fs::read_to_string(&output_path)?;
        if current != generated {
            return Err(format!(
                "{} is stale; run apple/macos/generate-headers.sh",
                output_path.display()
            )
            .into());
        }

        return Ok(());
    }

    fs::write(output_path, generated)?;
    Ok(())
}

fn generate_c_header(repo_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let config = cbindgen::Config::from_file(repo_root.join("cbindgen.toml"))?;
    let bindings = cbindgen::Builder::new()
        .with_crate(repo_root)
        .with_config(config)
        .generate()?;

    let scratch_dir = repo_root.join("target/xtask");
    fs::create_dir_all(&scratch_dir)?;
    let scratch_header = scratch_dir.join("ashen_frontier.h");
    bindings.write_to_file(&scratch_header);
    Ok(fs::read_to_string(scratch_header)?)
}

fn repo_root() -> io::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("xtask manifest has no parent directory"))
}
