## Problem Statement

Orca currently helps agents manage workspaces, sync changes, and run critique flows, but it does not provide a lightweight built-in way to manage implementation issues inside a coding project. This forces agents to keep issue state in ad hoc notes, markdown files, or external tools that are not optimized for fast, deterministic CLI interaction.

The missing capability is a small, local, repository-scoped issue tracker that agents can use without leaving the terminal. The tracker needs to be simple, predictable, scriptable, and safe for concurrent agent usage. It also needs to preserve the issue-writing style already used in the project's issue-planning workflows, while avoiding a schema that is so rigid that it becomes expensive to evolve.

## Solution

Add an `orca issue` subcommand to the existing Orca CLI. The command family will manage issues stored in a SQLite database located under Orca's existing home directory. Issues will be scoped to the resolved repository, identified by repo-scoped zero-padded numeric IDs, and support lightweight workflow operations such as create, show, list, update, block, and unblock.

The issue model will keep a narrow set of structured fields for operational workflow state: identifier, repository scope, status, title, and dependency edges. Rich issue content will remain freeform in a body field so agents can capture planning detail without forcing frequent schema changes. Dependencies will be stored structurally for safe querying and mutation. The CLI will remain non-interactive and optimized for agent use, with text output by default and optional JSON on read commands.

## User Stories

1. As an agent working inside a repository, I want to create a new issue from the CLI, so that I can track implementation work without leaving the terminal.
2. As an agent, I want issue IDs to be short and repo-scoped, so that I can reference them easily in follow-up commands.
3. As an agent, I want issue IDs to render as zero-padded values, so that issue references are visually consistent and easy to scan.
4. As an agent, I want the CLI to infer the current repository automatically, so that I do not need to pass repo metadata on every command.
5. As an agent, I want to override the inferred repository with `--repo`, so that I can manage issues for another project without changing directories.
6. As a human operator, I want repository inference to fail clearly outside a git repository, so that issue commands do not silently target the wrong project.
7. As an agent, I want to create an issue with only a title, so that I can capture work quickly and fill in details later.
8. As an agent, I want new issues to default to `todo`, so that the most common creation flow is short.
9. As an agent, I want the issue body to be freeform text, so that I can capture detailed plans without being constrained by a rigid schema.
10. As an agent, I want to update only specific issue fields, so that I can patch state without rewriting the entire record.
11. As an agent, I want to clear the body explicitly, so that I can remove stale detail when the issue is simplified.
12. As an agent, I want to assign one or more blockers to an issue during creation, so that the initial workflow graph can be captured in one step.
13. As an agent, I want to replace the entire blocker set in a single update, so that I can rewrite dependency state deterministically.
14. As an agent, I want to add blockers incrementally, so that small graph edits do not require replacing the full set.
15. As an agent, I want to remove blockers incrementally, so that I can unblock work precisely.
16. As an agent, I want a dedicated `block` command, so that simple dependency edits are terse.
17. As an agent, I want a dedicated `unblock` command, so that removing dependency edges is equally terse.
18. As an agent, I want dependency mutations to be atomic, so that partial graph edits do not leave the issue tracker in an ambiguous state.
19. As an agent, I want the CLI to reject unknown blocker IDs, so that references remain trustworthy.
20. As an agent, I want the CLI to reject self-dependencies, so that I cannot create obviously invalid workflow state.
21. As an agent, I want the CLI to reject dependency cycles, so that blockers always represent a valid acyclic graph.
22. As an agent, I want dependencies to stay within the current repository, so that repo-scoped IDs remain unambiguous.
23. As a human operator, I want `orca issue list` to show all issues by default, so that I can inspect the entire repository issue set from one command.
24. As a human operator, I want list output to be compact and text-first, so that I can scan issue status quickly in the terminal.
25. As an agent, I want `orca issue list --json`, so that I can parse issue data programmatically without scraping text.
26. As an agent, I want to filter `list` by one or more statuses, so that I can narrow the working set to actionable categories.
27. As an agent, I want to filter `list` by a direct blocker ID, so that I can inspect which issues are immediately downstream of a given issue.
28. As a human operator, I want `orca issue show` to display issue metadata plus the raw body, so that I can inspect both workflow state and planning detail together.
29. As an agent, I want `show --json` to include blocker and reverse-dependency IDs, so that I can traverse the dependency graph without extra queries.
30. As a human operator, I want missing issue lookups to fail with a clear error message, so that I can distinguish "not found" from "empty result."
31. As an agent, I want `create` to print only the new issue ID, so that I can capture the created identifier directly in a script.
32. As an agent, I want `update`, `block`, and `unblock` to stay silent on success, so that command chaining remains clean.
33. As a human operator, I want mutation failures to print clear stderr messages and exit non-zero, so that operational mistakes are visible.
34. As an agent, I want issue creation to remain safe under concurrent writes, so that multiple agents can create issues without corrupting IDs.
35. As a maintainer, I want schema initialization to happen lazily, so that the feature works without a separate setup step.
36. As a maintainer, I want migrations to live inside the binary, so that schema management stays simple and versioned with the CLI.
37. As a maintainer, I want the schema to be intentionally narrow, so that the issue CLI remains easy to reason about and evolve.
38. As a maintainer, I want completed issues to remain in the database instead of being deleted, so that references remain stable and auditable.
39. As an agent, I want duplicate titles to be allowed, so that issue identity remains based on IDs rather than brittle title uniqueness rules.
40. As a maintainer, I want the first version to avoid generic search and cross-repo workflows, so that the core issue-management path is solid before expanding scope.

## Implementation Decisions

- Add a new `issue` namespace under the existing Orca CLI rather than creating a separate binary.
- Store issue data in a SQLite database at `~/.orca/issues.db`, alongside Orca's existing local state.
- Resolve repository scope from the current git repository by default, and allow `--repo <path>` on issue commands to target another repository.
- Use the repository's canonical root path as the true scope key to avoid collisions between repositories that share the same basename.
- Persist a denormalized repository display name alongside each issue row for rendering, without introducing a separate repository table in v1.
- Use `rusqlite` directly rather than introducing an ORM or heavier persistence abstraction.
- Manage schema versioning in the binary with `PRAGMA user_version`.
- Use an internal surrogate primary key for storage and foreign keys, while keeping a separate repo-scoped `local_id` as the human-facing identifier.
- Enforce uniqueness on the combination of repository scope and local issue ID.
- Keep local issue IDs monotonic per repository, never reused, and render them as four-digit zero-padded values in the CLI and JSON outputs.
- Store structured issue workflow fields only for repository scope, local ID, status, title, timestamps, and dependency edges.
- Keep the issue `body` as freeform text rather than attempting to normalize all planning fields from the issue-planning workflow into dedicated columns.
- Represent dependencies in a dedicated relationship table rather than embedding them in the issue row, so that blocker queries and validations stay simple.
- Restrict dependencies to issues within the same repository scope.
- Reject self-dependencies and reject any dependency mutation that would introduce a cycle in the directed blocker graph.
- Treat dependency edits as atomic operations. If any referenced blocker is invalid, missing, cross-repository, or cycle-inducing, the full mutation fails.
- Use a fixed status enum of `todo`, `in_progress`, `blocked`, and `done`.
- Keep status transitions permissive in v1 rather than enforcing a workflow state machine.
- Do not auto-change status when blockers are added or removed. Status remains explicitly controlled by the caller.
- Support the following command surface in v1: `list`, `show`, `create`, `update`, `block`, and `unblock`.
- Keep mutation commands single-target. Bulk issue mutation is out of scope, apart from supplying multiple blocker IDs against one target issue.
- Make `create` require a title, default status to `todo`, allow an empty body, and return only the new padded ID on stdout.
- Make `update` patch-like. Omitted fields remain unchanged.
- Allow `update` to replace blockers with `--blockers`, or patch blockers with `--add-blockers` and `--remove-blockers`.
- Reject mixed blocker mutation modes in one update call, so replacement and patch semantics cannot be combined ambiguously.
- Reject empty updates and reject no-op dependency mutations, so agent mistakes surface immediately.
- Support multiple blocker IDs in `block` and `unblock` while keeping each command scoped to one target issue.
- Default read commands to compact text output and support optional JSON output on `list` and `show`.
- Omit JSON output modes from mutation commands in v1.
- Make text `list` output compact and operational, showing issue ID, status, title, and blocker IDs.
- Sort list results by repo-scoped issue ID ascending.
- Include completed issues in unfiltered list output.
- Support repeated `--status` filters on `list`.
- Support `--blocked-by <id>` on `list` with direct-edge semantics only, not transitive graph traversal.
- Treat missing IDs in `list --blocked-by` as an error rather than as an empty result.
- Make text `show` render issue metadata, direct blockers, reverse dependencies, and the raw issue body.
- Make JSON dependency fields render zero-padded string IDs rather than raw integers.
- Include repository metadata in `show --json` but not in `list --json`, since list is already scoped to a single repository.

## Testing Decisions

- Good tests should validate external behavior and command contracts, not internal implementation details such as the exact SQL shape or helper decomposition.
- The database layer should be tested through observable issue operations: creation, lookup, update behavior, dependency mutation, ID allocation, cycle rejection, repository scoping, and filtering behavior.
- The repository resolution layer should be tested through user-visible behavior: current-repo inference, `--repo` override handling, failure outside a git repository, and normalization to a canonical repo root.
- The output layer should be tested at the contract level: text output shape for list and show, JSON field presence and ID formatting, silent mutation behavior, and error-message handling for invalid operations.
- The concurrency-sensitive issue creation path should be tested through transactional behavior and uniqueness guarantees rather than through direct implementation hooks.
- Dependency graph validation should be tested through scenarios that exercise missing blockers, self-dependencies, same-repo constraints, reverse dependencies, atomic failure, and cycle detection.
- Command-line parsing should be tested through supported flag combinations and rejected combinations, especially around repeated `--status` filters and mutually exclusive blocker mutation modes.
- Prior art exists in the current CLI's integration-style tests that use temporary repositories, temporary Orca state directories, and command-level behavior checks. The issue CLI should follow that style rather than introducing a substantially different testing approach unless implementation pressure clearly requires it.

## Out of Scope

- Hard deletion of issues.
- Cross-repository dependency edges.
- Generic full-text search over titles or bodies.
- Arbitrary query-language-style filtering.
- Automatic readiness or prioritization commands.
- Automatic status changes derived from blockers.
- Interactive prompts, editors, or body-file workflows.
- Mutation-command JSON output.
- Title uniqueness constraints.
- Reusing issue IDs after completion or correction.
- Markdown file mirroring to an `/issues` directory in v1.
- Broader project-management concepts such as assignees, labels, milestones, priorities, due dates, or comments.

## Further Notes

- The goal of this feature is not to build a full issue tracker. The goal is to give agents a deterministic local workflow primitive that fits naturally into Orca's existing CLI model.
- The schema is intentionally narrow because the most valuable part of the feature is operational reliability: stable IDs, valid dependency edges, repository scoping, and scriptable read/write behavior.
- Freeform bodies deliberately preserve flexibility. If the project later converges on a more opinionated issue-body template, that can be layered on top of the same storage model without redesigning the core operational fields.
