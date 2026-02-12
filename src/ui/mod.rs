pub mod banner;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Ui;

impl Ui {
    pub fn header(msg: &str) {
        let style = Style::new().bold().cyan();
        println!("{}", style.apply_to(msg));
    }

    pub fn success(msg: &str) {
        let style = Style::new().green().bold();
        println!("{} {}", style.apply_to("✓"), msg);
    }

    pub fn error(msg: &str) {
        let style = Style::new().red().bold();
        eprintln!("{} {}", style.apply_to("✗"), msg);
    }

    pub fn warning(msg: &str) {
        let style = Style::new().yellow().bold();
        println!("{} {}", style.apply_to("!"), msg);
    }

    pub fn info(msg: &str) {
        let style = Style::new().dim();
        println!("  {}", style.apply_to(msg));
    }

    pub fn step(n: usize, total: usize, msg: &str) {
        let style = Style::new().bold();
        let dim = Style::new().dim();
        println!(
            "{} {}",
            style.apply_to(format!("[{}/{}]", n, total)),
            dim.apply_to(msg)
        );
    }

    pub fn spinner(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
                .template("{spinner:.cyan} {msg}")
                .expect("invalid spinner template"),
        );
        pb.set_message(msg.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    pub fn label_value(label: &str, value: &str) {
        let label_style = Style::new().bold();
        println!("  {}: {}", label_style.apply_to(label), value);
    }
}
