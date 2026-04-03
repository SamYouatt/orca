use std::path::Path;
use std::process::{Command, Stdio};

use crate::config;
use crate::theme;

pub struct ScriptContext<'a> {
    pub workspace_name: &'a str,
    pub branch_name: &'a str,
    pub workspace_path: &'a Path,
}

pub fn run_setup_scripts(base_dir: &Path, repo: &Path, ctx: &ScriptContext) {
    let settings = config::load_global_settings(base_dir);
    let project = config::load_project_config(repo);

    if let Some(script) = config::resolve_script(settings.setup.as_ref(), base_dir) {
        run_script(&script, ctx);
    }

    if let Some(script) = config::resolve_script(project.setup.as_ref(), repo) {
        run_script(&script, ctx);
    }
}

fn run_script(script: &Path, ctx: &ScriptContext) {
    if !script.exists() {
        eprintln!(
            "{} setup script not found: {}",
            theme::yellow("warning:"),
            script.display()
        );
        return;
    }

    let result = Command::new(script)
        .current_dir(ctx.workspace_path)
        .env("ORCA_WORKSPACE_NAME", ctx.workspace_name)
        .env("ORCA_BRANCH_NAME", ctx.branch_name)
        .env("ORCA_WORKSPACE_PATH", ctx.workspace_path)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match result {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!(
                "{} setup script exited with status {}: {}",
                theme::red("error:"),
                status.code().unwrap_or(-1),
                script.display()
            );
        }
        Err(e) => {
            eprintln!(
                "{} failed to run setup script {}: {}",
                theme::red("error:"),
                script.display(),
                e
            );
        }
    }
}
