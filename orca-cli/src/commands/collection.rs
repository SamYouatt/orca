use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;

use crate::collection::{self, CatchRecord};
use crate::names::{self, Rarity};
use crate::theme;

const CONTENT_WIDTH: usize = 42;
const COL_WIDTH: usize = 20;
const WHITE_BG: &str = "\x1b[48;2;255;255;255m";
const RESET: &str = "\x1b[0m";
const RESET_WITH_BG: &str = "\x1b[0m\x1b[48;2;255;255;255m";

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

fn with_white_bg(s: &str, enabled: bool) -> String {
    if !enabled {
        return s.to_string();
    }
    let patched = s.replace(RESET, RESET_WITH_BG);
    format!("{WHITE_BG}{patched}{RESET}")
}

fn card_border(left: &str, right: &str, color_bg: bool) {
    let border = format!(
        "{}{}{}",
        theme::black(left),
        theme::black(&"━".repeat(CONTENT_WIDTH + 2)),
        theme::black(right)
    );
    println!("  {}", with_white_bg(&border.to_string(), color_bg));
}

fn card_line(content: &str, color_bg: bool) {
    let vis = visible_len(content);
    let pad = CONTENT_WIDTH.saturating_sub(vis);
    let inner = format!(
        "{} {}{} {}",
        theme::black("┃"),
        content,
        " ".repeat(pad),
        theme::black("┃")
    );
    println!("  {}", with_white_bg(&inner.to_string(), color_bg));
}

fn card_empty(color_bg: bool) {
    card_line("", color_bg);
}

fn bar(width: usize, caught: usize, total: usize, rarity: Option<Rarity>) -> String {
    let filled = if total == 0 {
        0
    } else {
        ((caught * width) / total).min(width)
    };
    let empty = width - filled;
    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);
    let colored_fill = match rarity {
        Some(r) => rarity_color(&filled_str, r),
        None => theme::teal(&filled_str).to_string(),
    };
    format!("{}{}", colored_fill, theme::grey(&empty_str))
}

fn rarity_color(s: &str, rarity: Rarity) -> String {
    match rarity {
        Rarity::Common => theme::dark_grey(s),
        Rarity::Rare => theme::blue(s),
        Rarity::Epic => theme::purple(s),
        Rarity::Legendary => theme::gold(s),
    }
    .to_string()
}

pub fn collection(base_dir: &Path) -> Result<()> {
    let color_bg = std::io::stdout().is_terminal();
    let col = collection::load(base_dir);
    let total = names::total_fish_count();
    let caught_total = col.catches.len();

    println!();
    card_border("┏", "┓", color_bg);

    let word = "Collection";
    let remaining = CONTENT_WIDTH - word.len() - 2;
    let left_tilde = remaining / 2;
    let right_tilde = remaining - left_tilde;
    card_line(
        &format!(
            "{} {} {}",
            theme::light_grey(&"~".repeat(left_tilde)),
            theme::black_bold(word),
            theme::light_grey(&"~".repeat(right_tilde)),
        ),
        color_bg,
    );

    card_empty(color_bg);

    let count_text = format!("{}/{}", caught_total, total);
    let bar_width = CONTENT_WIDTH - count_text.len() - 1;
    let overall = bar(bar_width, caught_total, total, None);
    card_line(
        &format!("{} {}", theme::black(&count_text), overall),
        color_bg,
    );

    card_empty(color_bg);

    for (left_r, right_r) in [
        (Rarity::Common, Rarity::Rare),
        (Rarity::Epic, Rarity::Legendary),
    ] {
        let left_fish = names::fish_for_rarity(left_r);
        let right_fish = names::fish_for_rarity(right_r);
        let left_caught = left_fish
            .iter()
            .filter(|f| col.catches.contains_key(**f))
            .count();
        let right_caught = right_fish
            .iter()
            .filter(|f| col.catches.contains_key(**f))
            .count();

        let left_label = format!("{} {}/{}", left_r.label(), left_caught, left_fish.len());
        let right_label = format!("{} {}/{}", right_r.label(), right_caught, right_fish.len());
        let cw = COL_WIDTH;
        card_line(
            &format!(
                "{}  {}",
                rarity_color(&format!("{:<cw$}", left_label), left_r),
                rarity_color(&format!("{:<cw$}", right_label), right_r),
            ),
            color_bg,
        );

        card_line(
            &format!(
                "{}  {}",
                bar(COL_WIDTH, left_caught, left_fish.len(), Some(left_r)),
                bar(COL_WIDTH, right_caught, right_fish.len(), Some(right_r)),
            ),
            color_bg,
        );

        card_empty(color_bg);
    }

    let mut all_catches: Vec<(&str, &CatchRecord)> = col
        .catches
        .iter()
        .map(|(name, record)| (name.as_str(), record))
        .collect();

    let empty_msg = theme::grey("start using orca to catch some fish").to_string();

    if all_catches.is_empty() {
        card_line(&theme::black_bold("Rarest catches").to_string(), color_bg);
        card_line(&empty_msg, color_bg);
        card_empty(color_bg);
        card_line(&theme::black_bold("Recent catches").to_string(), color_bg);
        card_line(&empty_msg, color_bg);
    } else {
        let mut by_rarity = all_catches.clone();
        by_rarity.sort_by(|a, b| {
            b.1.rarity
                .cmp(&a.1.rarity)
                .then(b.1.caught_at.cmp(&a.1.caught_at))
        });
        by_rarity.truncate(5);

        card_line(&theme::black_bold("Rarest catches").to_string(), color_bg);
        print_catches(&by_rarity, color_bg);
        card_empty(color_bg);

        all_catches.sort_by(|a, b| b.1.caught_at.cmp(&a.1.caught_at));
        all_catches.truncate(5);

        card_line(&theme::black_bold("Recent catches").to_string(), color_bg);
        print_catches(&all_catches, color_bg);
    }

    card_border("┗", "┛", color_bg);
    println!();

    Ok(())
}

fn print_catches(catches: &[(&str, &CatchRecord)], color_bg: bool) {
    let repo_w = catches.iter().map(|(_, r)| r.repo.len()).max().unwrap_or(0);
    let right_w = repo_w + 2 + 8;

    for (name, record) in catches {
        let date = record.caught_at.format("%d·%m·%y").to_string();
        let right_colored = format!(
            "{}  {}",
            theme::grey(&format!("{:>rw$}", record.repo, rw = repo_w)),
            theme::grey(&date),
        );
        let gap = CONTENT_WIDTH.saturating_sub(name.len() + right_w);
        let colored_name = rarity_color(name, record.rarity);
        card_line(
            &format!("{}{}{}", colored_name, " ".repeat(gap), right_colored),
            color_bg,
        );
    }
}
