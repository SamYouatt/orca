# Orca

Orca is an attempt to create a parallel worktree based dev flow around composable pieces.

A lot of the functionality is based around [Conductor](https://www.conductor.build/).

## Pieces

- `orca` cli, the core cli that drives worktree functionality
- `orca` tui, a simple tui that can be pinned in zellij pane for status overview
- `zellij` for isolated sessions, session switching, and overview pane pinning
- `plugin` the claude plugin that includes slash commands for features like PR generation
- `gh` cli for interactions with GitHub like reviewing comments
- `plannotator` for plan and edit review system

## Features

### Cli

- `orca new`: Create a new workspace from the current repo and initialise a worktree
- `orca ls`: List all active workspaces
- `orca rm`: Remove a workspace

### Workspaces

Workspaces are created at `~/.orca/workspaces`
