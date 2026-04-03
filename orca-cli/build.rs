use std::process::Command;

fn main() {
    let review_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../orca-review");

    println!("cargo:rerun-if-changed={}",review_dir.join("ui").display());
    println!("cargo:rerun-if-changed={}",review_dir.join("package.json").display());
    println!("cargo:rerun-if-changed={}",review_dir.join("vite.config.ts").display());
    println!("cargo:rerun-if-changed={}",review_dir.join("tsconfig.json").display());

    let install = Command::new("bun")
        .arg("install")
        .arg("--frozen-lockfile")
        .current_dir(&review_dir)
        .status()
        .expect("failed to run bun install for orca-review");

    if !install.success() {
        panic!("orca-review bun install failed");
    }

    let status = Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir(&review_dir)
        .status()
        .expect("failed to run bun build for orca-review");

    if !status.success() {
        panic!("orca-review build failed");
    }
}
