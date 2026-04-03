# Orca

Orca is an attempt to create a parallel worktree based dev flow around composable pieces.

A lot of the functionality is inspired by [Conductor](https://www.conductor.build/).

## Pieces

- `orca` cli, the core cli that drives worktree functionality

## Workflow

1. Start a new workspace from the current repo with `orca new --branch my-branch-name`
2. Navigate to its workspace at `~/.orca/workspaces`
3. Optionally start `orca watch` for automatic teardown on PR merge
4. Check the status of other workspaces with `orca status` and jump to them if you need to work on them
4. Complete your work and run `orca complete` to teardown the workspace

## Configuration

Orca stores configuration in two places:
- Overarching configuration is stored in `$HOME/.orca/settings.json`
- Per project configuration is stored in `orca.json`

### Options

#### Setup

You may need to perform actions when workspaces are created and orca supports this by executing your provided script.

Scripts will be sourced from both the root orca configuration and project configuration. Root orca configuration always runs and completes before beginning project configuration.

```json
{
  "setup": {
    "script": "./my-script.sh",
  }
}
```

Setup scripts will be provided with the below environment variables set:
- `$ORCA_WORKSPACE_NAME`: the generated workspace name that orca created or `--name` if specified
- `$ORCA_BRANCH_NAME`: the branch name at time of creation, either the same as the workspace name or the provided `--branch` name
- `$ORCA_WORKSPACE_PATH`: the path to the workspace
