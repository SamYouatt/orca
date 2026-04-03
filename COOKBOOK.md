# Cookbook

Recipes for integrating orca with other tools.

## cmux

Open each new workspace in its own cmux workspace automatically.

**`~/.orca/settings.json`**
```json
{
  "setup": {
    "script": "/path/to/setup.sh"
  }
}
```

**`setup.sh`**
```bash
#!/bin/bash
cmux new-workspace --name "$ORCA_WORKSPACE_NAME" --cwd "$ORCA_WORKSPACE_PATH"
```

This gives you a dedicated cmux workspace per orca workspace. From there you can split panes, run agents, and use `cmux send` to drive them programmatically.

### Launching an agent in the workspace

```bash
#!/bin/bash
WS=$(cmux new-workspace --name "$ORCA_WORKSPACE_NAME" --cwd "$ORCA_WORKSPACE_PATH")
cmux send --workspace "$WS" "claude --dangerously-skip-permissions"
```

### Teardown

Close the cmux workspace when the orca workspace is removed:

**`~/.orca/settings.json`**
```json
{
  "setup": {
    "script": "/path/to/setup.sh"
  },
  "teardown": {
    "script": "/path/to/teardown.sh"
  }
}
```

**`teardown.sh`**
```bash
#!/bin/bash
WS=$(cmux find-window "$ORCA_WORKSPACE_NAME" 2>/dev/null | head -1)
if [ -n "$WS" ]; then
  cmux close-workspace --workspace "$WS"
fi
```

## wezterm

Use wezterm's CLI to open workspaces in new tabs.

**`setup.sh`**
```bash
#!/bin/bash
wezterm cli spawn --cwd "$ORCA_WORKSPACE_PATH" --new-window
wezterm cli set-tab-title "$ORCA_WORKSPACE_NAME"
```

### Split layout with sync

Open a workspace in a new tab, then split the pane to run `orca sync` alongside it:

```bash
#!/bin/bash
PANE_ID=$(wezterm cli spawn --cwd "$ORCA_WORKSPACE_PATH")
wezterm cli set-tab-title --pane-id "$PANE_ID" "$ORCA_WORKSPACE_NAME"
SYNC_PANE=$(wezterm cli split-pane --right --percent 30 --cwd "$ORCA_WORKSPACE_PATH" --pane-id "$PANE_ID")
wezterm cli send-text --pane-id "$SYNC_PANE" "orca sync\n"
```

### Multiple agents side by side

Spawn a workspace with two panes for parallel agent work:

```bash
#!/bin/bash
PANE_ID=$(wezterm cli spawn --cwd "$ORCA_WORKSPACE_PATH")
wezterm cli set-tab-title --pane-id "$PANE_ID" "$ORCA_WORKSPACE_NAME"
wezterm cli split-pane --right --percent 50 --cwd "$ORCA_WORKSPACE_PATH" --pane-id "$PANE_ID"
```

## Per-project setup

Install dependencies automatically when a workspace is created for a specific project.

**`orca.json`** (in project root)
```json
{
  "setup": {
    "script": "./scripts/orca-setup.sh"
  }
}
```

**`scripts/orca-setup.sh`**
```bash
#!/bin/bash
npm install
cp .env.example .env
```

This runs after the global setup script, so your terminal integration opens first, then dependencies install inside it.

## Porcelain status for scripting

`orca status --porcelain` outputs machine-readable workspace status. Combine it with other tools:

```bash
# list workspace names only
orca status --porcelain | awk '{print $1}'

# find workspaces with open PRs
orca status --porcelain | grep 'PR #'
```
