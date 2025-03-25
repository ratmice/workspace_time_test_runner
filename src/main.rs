use relative_path::PathExt;
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(serde::Deserialize)]
struct CargoMetadata {
    workspace_root: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir().unwrap();
    let dir = "--dir".to_string();
    let mut args = vec!["run".to_string()];
    let env_vars = ["OUT_DIR", "CARGO_MANIFEST_DIR"];
    let output = Command::new("cargo").args(["metadata"]).output()?;
    let s = String::from_utf8(output.stdout)?;
    let metadata: CargoMetadata = serde_json::from_str(&s)?;

    let to_workspace = cwd
        .relative_to(Path::new(&metadata.workspace_root))?
        .to_path(".");

    for path in to_workspace.ancestors() {
        args.extend([dir.clone(), path.display().to_string()]);
    }

    // Add the paths in `ENV_VARS` --env converting them relative to the current directory
    // Add the --dir preopens too.
    for var in env_vars {
        if let Ok(env_dir) = std::env::var(var) {
            let rel = if Path::new(&env_dir) == cwd.clone() {
                ".".to_string()
            } else {
                Path::new(&env_dir).relative_to(cwd.clone())?.to_string()
            };
            args.extend(["--env".to_string(), format!("{var}={}", &rel.clone())]);
            args.extend([dir.clone(), rel.to_string()]);
        };
    }

    // Now add `--dir .`, even if this has been added previously,
    // the last one will set the current working directory
    args.extend([dir.clone(), ".".to_string()]);
    // cargo test will run the "runner" with args, skip argv[0]
    args.extend(std::env::args().skip(1));
    let mut child = Command::new("wasmtime")
        .args(args)
        .spawn()
        .expect("failed to execute process");
    child.wait().expect("command wasn't running");
    Ok(())
}
