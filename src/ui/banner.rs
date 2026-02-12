use console::Style;

pub fn print_banner() {
    let style = Style::new().bold().magenta();
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{}",
        style.apply_to(format!(
            r#"
        _ _
 __   _(_) |__   ___
 \ \ / / | '_ \ / _ \
  \ V /| | |_) |  __/
   \_/ |_|_.__/ \___|  v{}

  AI-powered package manager
"#,
            version
        ))
    );
}
