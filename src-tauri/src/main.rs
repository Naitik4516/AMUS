// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Dual-mode: CLI vs GUI
    if amus_lib::cli::is_cli_invocation(&args) {
        std::process::exit(match amus_lib::cli::run_cli(args) {
            std::process::ExitCode::SUCCESS => 0,
            _ => 1,
        });
    }

    amus_lib::run()
}
