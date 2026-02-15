#![forbid(unsafe_code)]

use std::io;
use std::path::PathBuf;

use clap::Parser;

use vaultura::config::AppConfig;
use vaultura::ui::app::App;

#[derive(Parser)]
#[command(name = "vaultura", version, about = "A secure terminal-based password manager")]
struct Cli {
    /// Path to the vault file
    #[arg(short, long)]
    vault: Option<PathBuf>,

    /// Path to the config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut config = if let Some(ref config_path) = cli.config {
        AppConfig::load_from(config_path).unwrap_or_else(|e| {
            eprintln!("Warning: could not load config: {e}");
            AppConfig::default()
        })
    } else {
        AppConfig::load().unwrap_or_else(|_| AppConfig::default())
    };

    if let Some(vault_path) = cli.vault {
        config.vault_path = vault_path;
    }

    // Install panic hook that restores terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        original_hook(panic_info);
    }));

    let mut terminal = ratatui::init();
    let result = App::new(config).run(&mut terminal);
    ratatui::restore();
    result
}
