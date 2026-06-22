## Problem Statement

Orca critique currently gives reviewers two useful diff scopes: uncommitted changes and all changes on the current branch. That is enough for reviewing a working tree or a whole branch, but it is awkward when the branch contains several commits and the reviewer wants to inspect one commit at a time.

The missing capability is a commit-scoped critique mode. The user should be able to stay in the existing browser review flow, choose a commit from the current branch, and review exactly the diff introduced by that commit without manually copying SHAs or restarting `orca critique`.

## Solution

Add a third diff mode to Orca critique: commit. In commit mode, the review header shows a dropdown of commits from the current branch. Choosing a commit reloads the diff viewer with the patch introduced by that commit, while preserving the existing review affordances for file navigation, split or unified viewing, annotations, and feedback submission.

The initial critique experience should remain familiar. Orca still opens on uncommitted changes by default, still supports switching to the branch diff, and adds commit selection as a peer mode rather than a separate command. The commit dropdown should list branch commits in a compact, scannable format using abbreviated SHA, commit subject, and enough ordering context for the user to confidently pick the intended commit.

## User Stories

1. As an Orca critique user, I want a commit diff mode, so that I can review one commit from my branch without leaving the browser review UI.
2. As an Orca critique user, I want commit mode to sit alongside uncommitted and branch modes, so that all diff scopes feel like one workflow.
3. As an Orca critique user, I want to select a commit from a dropdown, so that I do not need to copy or type commit SHAs manually.
4. As an Orca critique user, I want the dropdown to show commits from the current branch, so that the list is focused on the work I am reviewing.
5. As an Orca critique user, I want the dropdown to exclude commits already on the default branch, so that the list does not include unrelated repository history.
6. As an Orca critique user, I want commits ordered predictably from newest to oldest, so that recent work is easy to find.
7. As an Orca critique user, I want each commit option to show an abbreviated SHA, so that I can match it to terminal output or git history.
8. As an Orca critique user, I want each commit option to show the commit subject, so that I can identify the commit by intent rather than by SHA alone.
9. As an Orca critique user, I want commit mode to show the selected commit clearly in the header, so that I always know what scope I am reviewing.
10. As an Orca critique user, I want switching from uncommitted mode to commit mode to load the selected commit's patch, so that the file tree and diff viewer reflect only that commit.
11. As an Orca critique user, I want switching from branch mode to commit mode to load only the selected commit, so that the whole-branch diff does not leak into the focused review.
12. As an Orca critique user, I want switching between two selected commits to reload the file tree, so that each commit has its own accurate list of changed files.
13. As an Orca critique user, I want switching out of commit mode to branch mode to restore the current whole-branch diff, so that I can compare focused and broad review scopes.
14. As an Orca critique user, I want switching out of commit mode to uncommitted mode to restore the current working tree diff, so that I can return to local changes quickly.
15. As an Orca critique user, I want annotations to clear when I change diff scopes, so that comments from one commit are not accidentally submitted against another scope.
16. As an Orca critique user, I want viewed-file state to clear when I change commits, so that the review progress reflects the selected commit.
17. As an Orca critique user, I want collapsed file-tree state to reset when I change commits, so that the file tree opens cleanly for the new patch.
18. As an Orca critique user, I want the active file to reset to the first changed file after selecting a commit, so that I can begin reviewing immediately.
19. As an Orca critique user, I want an empty selected commit to show the existing no-changes state, so that unusual commits are handled without breaking the UI.
20. As an Orca critique user, I want binary or unreadable files to behave consistently with existing diff behavior, so that commit mode does not introduce special cases in the review UI.
21. As an Orca critique user, I want renamed files in a commit to render with old and new paths where possible, so that commit-scoped review is as informative as branch review.
22. As an Orca critique user, I want added and deleted line counts to remain accurate in commit mode, so that file tree summary information stays trustworthy.
23. As an Orca critique user, I want split and unified view preferences to keep working when I select a commit, so that changing diff scope does not change my viewing style.
24. As an Orca critique user, I want the feedback bar to work the same way in commit mode, so that submitting or copying review feedback does not require a new workflow.
25. As an Orca critique user, I want feedback generated from commit mode to contain the annotations I made on that commit, so that the recipient gets focused review comments.
26. As an Orca critique user, I want errors while loading a commit diff to appear in the review UI, so that I can understand why the selected commit cannot be shown.
27. As an Orca critique user, I want the commit dropdown disabled while a diff switch is in progress, so that I cannot accidentally issue conflicting requests.
28. As an Orca critique user, I want the diff mode controls disabled while switching, so that the UI does not race between scopes.
29. As an Orca critique user, I want commit mode to recover after an error by letting me choose another diff scope, so that one bad selection does not end the review session.
30. As an Orca critique user, I want the selected commit to remain selected after its diff loads, so that the UI reflects the server's current diff state.
31. As an Orca critique user, I want the server to validate selected commit SHAs, so that malformed or stale browser requests cannot ask Orca to diff arbitrary invalid input.
32. As an Orca critique user, I want stale commit selections to fail clearly if history changes while the review server is running, so that I do not silently review the wrong patch.
33. As an Orca critique user, I want the commit list to be loaded from the same local repository session as the diff, so that the UI reflects the exact branch being critiqued.
34. As an Orca critique user, I want commit mode to work without network access, so that local critique remains usable offline.
35. As an Orca critique user, I want commit mode to use the local default branch detection already used by branch mode, so that the definition of current branch work is consistent.
36. As an Orca critique user, I want a clear empty state when there are no branch commits to select, so that I know commit mode is unavailable rather than broken.
37. As an Orca critique user, I want the UI to avoid showing a useless dropdown when there are no selectable commits, so that the header remains clean.
38. As an Orca critique user, I want the dropdown to fit in the compact critique header, so that it does not crowd the existing view-style and diff-mode controls.
39. As an Orca critique user, I want long commit subjects truncated accessibly, so that the header remains usable on narrow screens.
40. As an Orca critique user, I want enough tooltip or title text to inspect a truncated commit subject, so that truncation does not hide important context.
41. As an Orca critique user, I want the current selected diff label to be clear in all modes, so that I can distinguish "Uncommitted", "Branch", and a specific commit.
42. As an Orca critique user, I want commit mode to preserve Orca critique's single-page browser flow, so that the feature does not require navigation or new pages.
43. As a maintainer, I want commit enumeration to be a small backend capability, so that the frontend does not need to understand git history rules.
44. As a maintainer, I want diff generation for uncommitted, branch, and commit scopes to share one backend abstraction, so that adding the third scope does not duplicate patch parsing behavior.
45. As a maintainer, I want commit diff requests to carry both a diff type and commit identifier, so that the API contract is explicit and extensible.
46. As a maintainer, I want server responses to include the selected commit metadata when commit mode is active, so that the UI can render without making assumptions.
47. As a maintainer, I want the initial diff response to include available commit options, so that the frontend can render the dropdown without an extra round trip.
48. As a maintainer, I want diff-switch responses to include refreshed commit options, so that the UI stays correct if the branch changes while the server is open.
49. As a maintainer, I want the server to be the source of truth for current diff state, so that browser state cannot drift from the patch being shown.
50. As a maintainer, I want tests around commit list and commit patch generation, so that the git behavior is stable across future critique changes.

## Implementation Decisions

- Add `commit` as a third critique diff type alongside `uncommitted` and `branch`.
- Keep `orca critique` as the only command surface for this feature. Do not add a separate CLI flag or subcommand for launching directly into commit mode in v1.
- Continue opening critique on uncommitted changes by default.
- Define a selectable branch commit as a commit reachable from `HEAD` and not reachable from the merge base with the detected default branch.
- Use the same default branch detection path as the existing branch diff mode, so branch mode and commit mode agree on what counts as branch work.
- List commit options newest-first.
- Represent each commit option with full SHA, abbreviated SHA, subject, and selected-state metadata returned by the backend.
- Render commit options in the UI using abbreviated SHA plus subject.
- Treat the selected commit diff as the patch introduced by that commit against its first parent.
- For merge commits, v1 should either render the first-parent patch with a clear label or exclude merge commits from the dropdown with a clear implementation comment. The preferred v1 behavior is first-parent patch because it preserves a simple "what this commit introduced" model.
- Add commit identity to the diff-switch request contract. Non-commit modes do not require a commit identifier.
- Reject commit diff requests without a commit identifier.
- Reject commit diff requests when the commit identifier does not resolve to a selectable current-branch commit.
- Return a clear API error when commit enumeration cannot find a merge base with the default branch.
- Return a clear API error when a selected commit cannot be diffed.
- Include available commit options in diff API responses so the frontend can render or update the dropdown from one server response.
- Include selected commit metadata in diff API responses when commit mode is active.
- Keep the existing `gitRef` response concept, but make commit mode set it to a human-readable selected-commit label.
- Update the frontend diff type model to include `commit`.
- Replace the two-option diff toggle with a control that still makes uncommitted and branch scopes obvious while allowing commit mode to expose a dropdown.
- The commit dropdown should only be active when commit mode is selected and at least one commit option exists.
- The UI should reset annotations, viewed files, collapsed directories, and active file whenever the server returns a new diff scope or selected commit.
- Keep file parsing and file tree behavior shared across all diff modes.
- Keep feedback submission unchanged. Feedback remains a list of annotations; the selected diff scope affects what the user saw, not the feedback payload contract.
- Extract or formalize a deep diff-source module behind a small interface that can list available commit options, resolve a requested diff scope, produce the raw patch, and provide old/new file contents for that scope.
- Keep git command execution behind backend code. The browser should never construct git commands or infer branch history locally.
- Avoid persisting selected commit state outside the in-memory review server session in v1.
- Avoid adding new database or schema state for this feature.
- Keep the feature local-only and offline-capable.

## Testing Decisions

- Good tests should validate externally visible behavior and contracts, not internal helper layout. Tests should prove that selecting a diff scope produces the expected patch, metadata, file contents, and error behavior.
- The diff-source module should be tested with temporary git repositories that create a default branch, feature branch commits, uncommitted changes, untracked files, and merge-base scenarios.
- Commit enumeration should be tested for newest-first ordering, exclusion of default-branch commits, commit metadata shape, and behavior when no branch-only commits exist.
- Commit patch generation should be tested by creating multiple commits that touch distinct files and asserting that selecting one commit returns only that commit's changes.
- Commit validation should be tested by requesting malformed SHAs, valid SHAs outside the selectable branch range, and stale SHAs after branch history changes.
- Branch diff behavior should continue to be tested to ensure adding commit mode does not change the existing merge-base branch patch.
- Uncommitted diff behavior should continue to be tested to ensure adding commit mode does not change working tree and untracked file handling.
- File-content loading should be tested for commit mode, especially that old content comes from the selected commit's parent and new content comes from the selected commit where possible.
- Server API behavior should be tested at the request/response contract level: initial diff response includes commit options, commit switch requires a commit ID, invalid switches return errors, and successful switches update diff type, selected commit, git ref, patch, and files.
- Frontend behavior should be covered at the component or app interaction level if a test harness is introduced: switching to commit mode, choosing a commit, disabled switching state, empty commit list state, and reset of annotation/review progress on scope change.
- Prior art exists in the CLI integration tests that use temporary repositories and command-level behavior checks. Backend tests for commit diff selection should follow that style.
- The review app currently has no dedicated frontend test setup. If frontend tests are added, keep them focused on user-visible state transitions rather than implementation details of React state.

## Out of Scope

- Launching `orca critique` directly into a chosen commit from the CLI.
- Persisting the last selected diff mode or commit between critique sessions.
- Reviewing an arbitrary commit outside the current branch's branch-only commit set.
- Reviewing arbitrary commit ranges.
- Comparing two user-selected commits.
- Editing, squashing, reordering, or otherwise mutating commits from the critique UI.
- Fetching remote history or relying on network access to populate the commit dropdown.
- Integrating with GitHub pull request commit lists.
- Changing the feedback payload format to include commit metadata.
- Adding a full git log browser.
- Adding search, filters, or keyboard navigation for very large commit lists in v1.
- Changing branch mode semantics.
- Changing uncommitted mode semantics.

## Further Notes

- The main product goal is review focus. The feature should make it cheap to inspect one logical commit while preserving the existing ability to review the whole branch or the working tree.
- The most important architectural choice is to keep commit selection as a backend diff-source concern. That gives the UI a simple list of choices and keeps git semantics testable in isolation.
- The spelling of the current UI label "Uncommited" should be corrected to "Uncommitted" while touching the diff mode control.
