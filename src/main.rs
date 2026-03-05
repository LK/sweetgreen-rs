use clap::Parser;
use sweetgreen_rs::cli::{Cli, run};
use sweetgreen_rs::debug::write_failure_report;
use sweetgreen_rs::error::SweetgreenError;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli).await {
        maybe_write_cli_error_report(&err);
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn maybe_write_cli_error_report(err: &SweetgreenError) {
    let report = write_failure_report(
        "cli_error",
        &serde_json::json!({
            "error_display": err.to_string(),
            "error_debug": format!("{err:?}"),
            "argv": std::env::args().collect::<Vec<_>>(),
            "cwd": std::env::current_dir()
                .ok()
                .and_then(|path| path.into_os_string().into_string().ok()),
        }),
    );

    if let Some(path) = report {
        eprintln!("debug report: {}", path.display());
    }
}
