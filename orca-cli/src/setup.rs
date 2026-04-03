use std::fmt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config;
use crate::theme;

pub struct ScriptContext<'a> {
    pub workspace_name: &'a str,
    pub branch_name: &'a str,
    pub workspace_path: &'a Path,
}

enum ScriptKind {
    Setup,
    Teardown,
}

impl fmt::Display for ScriptKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptKind::Setup => write!(f, "setup"),
            ScriptKind::Teardown => write!(f, "teardown"),
        }
    }
}

pub fn run_setup_scripts(base_dir: &Path, repo: &Path, ctx: &ScriptContext) {
    let settings = config::load_global_settings(base_dir);
    let project = config::load_project_config(repo);

    if let Some(script) = config::resolve_script(settings.setup.as_ref(), base_dir) {
        run_script(&script, ctx, ScriptKind::Setup);
    }

    if let Some(script) = config::resolve_script(project.setup.as_ref(), repo) {
        run_script(&script, ctx, ScriptKind::Setup);
    }
}

pub fn run_teardown_scripts(base_dir: &Path, repo: &Path, ctx: &ScriptContext) {
    let settings = config::load_global_settings(base_dir);
    let project = config::load_project_config(repo);

    if let Some(script) = config::resolve_script(project.teardown.as_ref(), repo) {
        run_script(&script, ctx, ScriptKind::Teardown);
    }

    if let Some(script) = config::resolve_script(settings.teardown.as_ref(), base_dir) {
        run_script(&script, ctx, ScriptKind::Teardown);
    }
}

fn run_script(script: &Path, ctx: &ScriptContext, kind: ScriptKind) {
    if !script.exists() {
        eprintln!(
            "{} {} script not found: {}",
            theme::yellow("warning:"),
            kind,
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
                "{} {} script exited with status {}: {}",
                theme::red("error:"),
                kind,
                status.code().unwrap_or(-1),
                script.display()
            );
        }
        Err(e) => {
            eprintln!(
                "{} failed to run {} script {}: {}",
                theme::red("error:"),
                kind,
                script.display(),
                e
            );
        }
    }
}
