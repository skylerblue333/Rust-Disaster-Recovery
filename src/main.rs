use sky_recovery::{build_manifest, verify_manifest, Manifest};
use std::env;
use std::fs;
use std::path::Path;

fn usage() {
    println!("sky-recovery create <root> <relative-path>...\nsky-recovery verify <manifest.json> <root>");
}

fn run() -> Result<i32, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && matches!(args[1].as_str(), "--help" | "-h") {
        usage();
        return Ok(0);
    }
    match args.get(1).map(String::as_str) {
        Some("create") if args.len() >= 4 => {
            let root = Path::new(&args[2]);
            let manifest = build_manifest(root, &args[3..])?;
            let json = serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
            println!("{json}");
            Ok(0)
        }
        Some("verify") if args.len() == 4 => {
            let raw = fs::read_to_string(&args[2]).map_err(|error| error.to_string())?;
            let manifest: Manifest = serde_json::from_str(&raw).map_err(|error| error.to_string())?;
            let report = verify_manifest(Path::new(&args[3]), &manifest);
            let json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
            println!("{json}");
            Ok(if report.is_ok() { 0 } else { 1 })
        }
        _ => {
            usage();
            Ok(2)
        }
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
