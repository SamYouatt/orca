use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{EventKind, RecursiveMode, Watcher, recommended_watcher};

use crate::{git, theme};

const DEBOUNCE_MS: u64 = 200;
const POLL_MS: u64 = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Root,
    Worktree,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PendingSide {
    One(Side),
    Both,
}

impl PendingSide {
    pub fn merge(self, other: Side) -> Self {
        match self {
            PendingSide::One(s) if s == other => self,
            _ => PendingSide::Both,
        }
    }

    pub fn has_root(self) -> bool {
        matches!(self, PendingSide::One(Side::Root) | PendingSide::Both)
    }

    pub fn has_worktree(self) -> bool {
        matches!(self, PendingSide::One(Side::Worktree) | PendingSide::Both)
    }
}

pub struct SyncState {
    pub pending: Mutex<HashMap<PathBuf, (Instant, PendingSide)>>,
    pub in_flight: Mutex<HashSet<PathBuf>>,
    pub root_written: Mutex<HashSet<PathBuf>>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            in_flight: Mutex::new(HashSet::new()),
            root_written: Mutex::new(HashSet::new()),
        }
    }
}

pub fn build_filter(root: &Path) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(root);
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }
    let filter = builder.build()?;
    Ok(filter)
}

pub fn is_ignored(filter: &Gitignore, root: &Path, path: &Path) -> bool {
    if path.starts_with(root.join(".git")) {
        return true;
    }
    filter
        .matched_path_or_any_parents(path, path.is_dir())
        .is_ignore()
}

pub fn relative_path(base: &Path, full: &Path) -> Option<PathBuf> {
    full.strip_prefix(base).ok().map(|p| p.to_path_buf())
}

fn files_identical(a: &Path, b: &Path) -> bool {
    let (Ok(ma), Ok(mb)) = (a.metadata(), b.metadata()) else {
        return false;
    };
    if ma.len() != mb.len() {
        return false;
    }
    match (fs::read(a), fs::read(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

pub fn copy_or_delete(src: &Path, dst: &Path, state: &SyncState) -> Result<bool> {
    if src.exists() && dst.exists() && files_identical(src, dst) {
        return Ok(false);
    }

    state.in_flight.lock().unwrap().insert(dst.to_path_buf());

    let result = if src.exists() {
        let mkdir = if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
        } else {
            Ok(())
        };
        mkdir.and_then(|_| fs::copy(src, dst).map(|_| ()))
    } else if dst.exists() {
        fs::remove_file(dst)
    } else {
        return Ok(false);
    };

    if let Err(ref e) = result {
        state.in_flight.lock().unwrap().remove(dst);
        return Err(anyhow::anyhow!("{}", e));
    }

    Ok(true)
}

pub fn sync_once(
    state: &SyncState,
    root: &Path,
    worktree: &Path,
    root_filter: &Gitignore,
    wt_filter: &Gitignore,
    verbose: bool,
) -> Vec<(PathBuf, Side)> {
    let ready: Vec<(PathBuf, PendingSide)> = {
        let mut pending = state.pending.lock().unwrap();
        let now = Instant::now();
        let debounce = Duration::from_millis(DEBOUNCE_MS);

        let ready_keys: Vec<PathBuf> = pending
            .iter()
            .filter(|(_, (ts, _))| now.duration_since(*ts) >= debounce)
            .map(|(k, _)| k.clone())
            .collect();

        ready_keys
            .into_iter()
            .filter_map(|k| pending.remove(&k).map(|(_ts, side)| (k, side)))
            .collect()
    };

    let mut synced = Vec::new();

    for (rel, pending_side) in ready {
        if pending_side.has_root() && pending_side.has_worktree() {
            let src = root.join(&rel);
            let dst = worktree.join(&rel);
            if !is_ignored(root_filter, root, &src) {
                match copy_or_delete(&src, &dst, state) {
                    Err(e) => {
                        eprintln!(
                            "  {} sync {} → worktree: {}",
                            theme::red("err"),
                            rel.display(),
                            e
                        );
                    }
                    Ok(true) => {
                        if verbose {
                            println!(
                                "  {} {} → worktree (conflict: root wins)",
                                theme::yellow("~"),
                                theme::grey(&rel.display().to_string()),
                            );
                        }
                        synced.push((rel, Side::Root));
                    }
                    Ok(false) => {}
                }
            }
            continue;
        }

        let side = if pending_side.has_root() {
            Side::Root
        } else {
            Side::Worktree
        };

        let (src_base, dst_base, filter) = match side {
            Side::Root => (root, worktree, root_filter),
            Side::Worktree => (worktree, root, wt_filter),
        };

        let src = src_base.join(&rel);
        if is_ignored(filter, src_base, &src) {
            continue;
        }

        let dst = dst_base.join(&rel);
        let direction = match side {
            Side::Root => "root → worktree",
            Side::Worktree => "worktree → root",
        };

        match copy_or_delete(&src, &dst, state) {
            Err(e) => {
                eprintln!(
                    "  {} sync {} {}: {}",
                    theme::red("err"),
                    rel.display(),
                    direction,
                    e
                );
            }
            Ok(true) => {
                if verbose {
                    let symbol = if src.exists() {
                        theme::green("→")
                    } else {
                        theme::red("✕")
                    };
                    println!(
                        "  {} {} {}",
                        symbol,
                        theme::grey(&rel.display().to_string()),
                        direction,
                    );
                }
                if side == Side::Worktree {
                    state.root_written.lock().unwrap().insert(rel.clone());
                }
                synced.push((rel, side));
            }
            Ok(false) => {}
        }
    }

    synced
}

fn cleanup_root(root: &Path, root_written: &HashSet<PathBuf>) {
    if root_written.is_empty() {
        return;
    }

    let rel_paths: Vec<String> = root_written
        .iter()
        .map(|p| p.display().to_string())
        .collect();

    let tracked_set = git::tracked_files(root, &rel_paths);

    let tracked: Vec<&str> = rel_paths
        .iter()
        .filter(|p| tracked_set.contains(p.as_str()))
        .map(|p| p.as_str())
        .collect();
    let untracked: Vec<&str> = rel_paths
        .iter()
        .filter(|p| !tracked_set.contains(p.as_str()))
        .map(|p| p.as_str())
        .collect();

    let mut restored = 0;

    if git::checkout_files(root, &tracked).is_ok() {
        restored += tracked.len();
    }

    for rel in &untracked {
        let path = root.join(rel);
        if path.exists() {
            let _ = fs::remove_file(&path);
            restored += 1;
        }
    }

    if restored > 0 {
        println!(
            "\n  {} restored {} files in root",
            theme::green("✓"),
            restored,
        );
    }
}

fn make_watcher(
    base: PathBuf,
    state: Arc<SyncState>,
    tx: mpsc::Sender<(PathBuf, Side)>,
    side: Side,
) -> notify::Result<impl Watcher> {
    let git_dir = base.join(".git");
    recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
                _ => return,
            }
            for path in event.paths {
                if path.starts_with(&git_dir) {
                    continue;
                }
                if state.in_flight.lock().unwrap().remove(&path) {
                    continue;
                }
                let _ = tx.send((path, side));
            }
        }
    })
}

fn enqueue_event(state: &SyncState, root: &Path, worktree: &Path, path: PathBuf, side: Side) {
    let rel = match side {
        Side::Root => relative_path(root, &path),
        Side::Worktree => relative_path(worktree, &path),
    };
    if let Some(rel) = rel {
        let mut pending = state.pending.lock().unwrap();
        let new_side = PendingSide::One(side);
        pending
            .entry(rel)
            .and_modify(|(ts, existing)| {
                *ts = Instant::now();
                *existing = existing.merge(side);
            })
            .or_insert((Instant::now(), new_side));
    }
}

pub fn run(root: &Path, worktree: &Path, verbose: bool) -> Result<()> {
    if !root.exists() {
        bail!("root repo not found: {}", root.display());
    }
    if !worktree.exists() {
        bail!("worktree not found: {}", worktree.display());
    }

    let root = &root.canonicalize()?;
    let worktree = &worktree.canonicalize()?;

    let root_filter = build_filter(root)?;
    let wt_filter = build_filter(worktree)?;

    let state = Arc::new(SyncState::new());
    let shutdown = Arc::new(AtomicBool::new(false));

    let shutdown_flag = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_flag.store(true, Ordering::SeqCst);
    })?;

    let (tx, rx) = mpsc::channel();

    let mut root_watcher = make_watcher(root.to_path_buf(), state.clone(), tx.clone(), Side::Root)?;
    let mut wt_watcher = make_watcher(worktree.to_path_buf(), state.clone(), tx, Side::Worktree)?;

    root_watcher.watch(root, RecursiveMode::Recursive)?;
    wt_watcher.watch(worktree, RecursiveMode::Recursive)?;

    let state_poller = state.clone();
    let root_poller = root.to_path_buf();
    let worktree_poller = worktree.to_path_buf();
    let shutdown_poller = shutdown.clone();
    thread::spawn(move || {
        while !shutdown_poller.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(POLL_MS));
            sync_once(
                &state_poller,
                &root_poller,
                &worktree_poller,
                &root_filter,
                &wt_filter,
                verbose,
            );
        }
    });

    while !shutdown.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok((path, side)) => {
                enqueue_event(&state, root, worktree, path, side);
                while let Ok((path, side)) = rx.try_recv() {
                    enqueue_event(&state, root, worktree, path, side);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let written = state.root_written.lock().unwrap().clone();
    cleanup_root(root, &written);

    drop(root_watcher);
    drop(wt_watcher);

    Ok(())
}
