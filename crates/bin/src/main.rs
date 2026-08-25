use clap::Parser;

fn main() {
    // Initialize tracing. Logs go to ~/.rsopencode/rsopencode.log so they
    // don't interfere with the TUI. Controlled by RUST_LOG env var.
    let log_dir = dirs::home_dir()
        .map(|h| h.join(".rsopencode"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let _ = std::fs::create_dir_all(&log_dir);
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("rsopencode.log"))
        .ok();
    if let Some(file) = log_file {
        use tracing_subscriber::{fmt, EnvFilter};
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        fmt()
            .with_env_filter(filter)
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false)
            .try_init()
            .ok();
    }

    rsopencode::i18n::init();

    let cli = rsopencode::cli::cli::Cli::parse();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let result = rt.block_on(rsopencode::cli::commands::run(cli));

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
