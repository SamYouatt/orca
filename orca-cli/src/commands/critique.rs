mod diff;
mod format;
mod server;
mod types;

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use diff::{get_default_branch, run_diff};
use format::format_feedback;
use server::ReviewServer;

pub fn critique(_base_dir: &Path) -> Result<()> {
    let default_branch = get_default_branch();
    let (initial_patch, initial_ref, initial_error) = run_diff("uncommitted", &default_branch);

    if initial_patch.is_empty() && initial_error.is_none() {
        println!("No changes to review.");
        return Ok(());
    }

    let server = ReviewServer::start(initial_patch, initial_ref, initial_error, default_branch)?;

    eprintln!("Review ready at {}", server.url);
    let _ = Command::new("open").arg(&server.url).spawn();

    let payload = server.wait_for_feedback()?;

    std::thread::sleep(std::time::Duration::from_millis(1500));

    println!("{}", format_feedback(&payload));

    Ok(())
}
