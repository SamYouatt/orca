# orca

A worktree-based toolkit for agentic development. Spin up isolated workspaces, monitor their status, sync files, and review changes — all from the CLI.

## Quick start

```bash
cd my-project

# create a workspace (creates a git worktree with a generated name)
orca new

# optionally specify a branch name
orca new --branch my-feature

# check on all your workspaces
orca status

# clean up when done
orca rm
```

Workspaces live in `~/.orca/workspaces/` and are backed by git worktrees.

## Install

### GitHub releases

Download a prebuilt binary from [releases](https://github.com/SamYouatt/orca/releases).

### From source

```bash
cargo install --path orca-cli
```

## Commands

### Workspace management

| Command | Description |
|---------|-------------|
| `orca new` | Create a new workspace |
| `orca ls` | List all workspaces with their repo, branch, and creation date |
| `orca status` | Show workspaces with git diff stats, upstream status, and PR info (requires `gh`) |
| `orca rm` | Remove and teardown one or more workspaces |

### Development

| Command | Description |
|---------|-------------|
| `orca sync` | Live bidirectional file sync between the root repo and a workspace. Respects `.gitignore`, debounces changes, and restores the root on exit |
| `orca critique` | Opens an interactive code review in the browser. Diffs your changes against the default branch and lets you annotate them |

## Claude Code plugin

Orca ships with a Claude Code plugin. Install it from the marketplace:

```
/plugin marketplace add SamYouatt/orca
/plugin install orca
```

### `/critique`

Opens an interactive code review in your browser where you can annotate the agent's changes. Your feedback is passed back to the agent as instructions to act on.

## Configuration

Orca reads configuration from two places:

- **Global**: `~/.orca/settings.json`
- **Per-project**: `orca.json` in the repo root

Both support `setup` and `teardown` blocks:

```json
{
  "setup": {
    "script": "./scripts/setup.sh"
  },
  "teardown": {
    "script": "./scripts/teardown.sh"
  }
}
```

Setup scripts run on `orca new`, teardown scripts run on `orca rm`. Global scripts run before/after project scripts respectively. Use `--no-script` to skip them.

Scripts receive these environment variables:

| Variable | Description |
|----------|-------------|
| `ORCA_WORKSPACE_NAME` | The generated workspace name |
| `ORCA_BRANCH_NAME` | The branch name (same as workspace name unless `--branch` was used) |
| `ORCA_WORKSPACE_PATH` | Absolute path to the workspace |

See [COOKBOOK.md](COOKBOOK.md) for setup script examples and terminal integrations.
