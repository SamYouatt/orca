use chrono::{Duration, Utc};
use rand::RngExt;
use rand::prelude::IndexedRandom;

use orca::collection::{self, CatchRecord, Collection};
use orca::names;

const REPOS: &[&str] = &["my-project", "api-server", "frontend", "infra", "docs"];

fn main() {
    let base_dir = orca::base_dir();
    let mut rng = rand::rng();
    let count: usize = rng.random_range(8..=30);
    let now = Utc::now();

    let mut col = Collection::default();
    let mut added = 0;

    for _ in 0..count {
        let catch = names::generate();

        if col.catches.contains_key(&catch.name) {
            continue;
        }

        let days_ago = rng.random_range(0..=60);
        let caught_at = now - Duration::days(days_ago);
        let repo = REPOS.choose(&mut rng).unwrap();

        col.catches.insert(
            catch.name,
            CatchRecord {
                rarity: catch.rarity,
                caught_at,
                repo: repo.to_string(),
            },
        );
        added += 1;
    }

    collection::save(&base_dir, &col).expect("failed to save collection");
    println!("seeded collection with {} fish", added);
}
