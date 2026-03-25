use colored::{ColoredString, Colorize};

struct Rgb(u8, u8, u8);

const WHITE: Rgb = Rgb(255, 255, 255);
const BLUE: Rgb = Rgb(56, 100, 180);
const TEAL: Rgb = Rgb(0, 172, 172);
const GREEN: Rgb = Rgb(40, 180, 100);
const RED: Rgb = Rgb(220, 60, 60);
const YELLOW: Rgb = Rgb(220, 170, 30);
const PURPLE: Rgb = Rgb(160, 90, 210);
const LIGHT_BLUE: Rgb = Rgb(100, 160, 240);
const GREY: Rgb = Rgb(140, 140, 140);

fn fg(s: &str, Rgb(r, g, b): &Rgb) -> ColoredString {
    s.truecolor(*r, *g, *b)
}

fn bg(s: &str, Rgb(fr, fg_val, fb): &Rgb, Rgb(br, bg_val, bb): &Rgb) -> ColoredString {
    s.truecolor(*fr, *fg_val, *fb)
        .on_truecolor(*br, *bg_val, *bb)
}

pub fn header(s: &str) -> ColoredString {
    bg(s, &WHITE, &BLUE).bold()
}

pub fn green(s: &str) -> ColoredString {
    fg(s, &GREEN)
}

pub fn red(s: &str) -> ColoredString {
    fg(s, &RED)
}

pub fn yellow(s: &str) -> ColoredString {
    fg(s, &YELLOW)
}

pub fn purple(s: &str) -> ColoredString {
    fg(s, &PURPLE)
}

pub fn blue(s: &str) -> ColoredString {
    fg(s, &BLUE)
}

pub fn teal(s: &str) -> ColoredString {
    fg(s, &TEAL)
}

pub fn light_blue(s: &str) -> ColoredString {
    fg(s, &LIGHT_BLUE)
}

pub fn grey(s: &str) -> ColoredString {
    fg(s, &GREY)
}
