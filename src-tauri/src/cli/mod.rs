mod args;
mod client;
mod dispatch;
mod format;
mod paths;
pub mod protocol;
mod server;

pub use dispatch::play_paths;
pub use server::{cleanup as cleanup_server, start as start_server};

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use args::{
    parse_search_query, parse_signed_number, search_kind_from_scope, Cli, Commands, LibraryCmd,
    PlaylistCmd, QueueCmd,
};
use protocol::{CliCommand, SearchKind};

pub fn is_cli_invocation(args: &[String]) -> bool {
    if args.len() <= 1 {
        return false;
    }
    let first = args[1].as_str();
    if first == "--gui" {
        return false;
    }
    true
}

pub fn run_cli(args: Vec<String>) -> ExitCode {
    attach_console();

    let cli = match Cli::try_parse_from(&args) {
        Ok(c) => c,
        Err(e) => {
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
    };

    match run_cli_inner(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli_inner(cli: Cli) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    if cli.command.is_none() {
        if cli.paths.is_empty() {
            return Err("no command given. Try `amus help`.".into());
        }
        let files = paths::collect_audio_paths(&cli.paths, &cwd)?;
        let abs: Vec<String> = files
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        return send_and_print(CliCommand::PlayPaths { paths: abs });
    }

    let command = cli.command.unwrap();

    match &command {
        Commands::Version => {
            println!("AMUS {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Commands::Info { path } => {
            let text = client::local_info(path)?;
            println!("{text}");
            return Ok(());
        }
        _ => {}
    }

    let cmd = command_to_protocol(command, &cwd)?;
    send_and_print(cmd)
}

fn send_and_print(cmd: CliCommand) -> Result<(), String> {
    let resp = client::send_command(cmd)?;
    if !resp.ok {
        return Err(resp.error.unwrap_or_else(|| "unknown error".into()));
    }
    if let Some(data) = resp.data {
        let text = format::format_data(&data);
        if !text.is_empty() {
            println!("{text}");
        }
    }
    Ok(())
}

fn command_to_protocol(command: Commands, cwd: &std::path::Path) -> Result<CliCommand, String> {
    Ok(match command {
        Commands::Play {
            search,
            paths: play_paths,
        } => {
            if let Some(q) = search {
                let (query, kind) = parse_search_query(&q);
                CliCommand::PlaySearch { query, kind }
            } else if !play_paths.is_empty() {
                let files = paths::collect_audio_paths(&play_paths, cwd)?;
                CliCommand::PlayPaths {
                    paths: files
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect(),
                }
            } else {
                CliCommand::Play
            }
        }
        Commands::Pause => CliCommand::Pause,
        Commands::Stop => CliCommand::Stop,
        Commands::Toggle => CliCommand::Toggle,
        Commands::Next => CliCommand::Next,
        Commands::Prev => CliCommand::Previous,
        Commands::Seek { position } => {
            let (value, relative) = parse_signed_number(&position)?;
            CliCommand::Seek { value, relative }
        }
        Commands::Volume { level } => {
            let (value, relative) = parse_signed_number(&level)?;
            CliCommand::Volume { value, relative }
        }
        Commands::Mute => CliCommand::Mute,
        Commands::Status => CliCommand::Status,
        Commands::Queue { action } => match action {
            QueueCmd::Add { search, paths: qp } => {
                if let Some(q) = search {
                    let (query, kind) = parse_search_query(&q);
                    CliCommand::QueueAddSearch { query, kind }
                } else if !qp.is_empty() {
                    let files = paths::collect_audio_paths(&qp, cwd)?;
                    CliCommand::QueueAddPaths {
                        paths: files
                            .into_iter()
                            .map(|p| p.to_string_lossy().into_owned())
                            .collect(),
                    }
                } else {
                    return Err("queue add requires paths or -s <query>".into());
                }
            }
            QueueCmd::Clear => CliCommand::QueueClear,
            QueueCmd::Shuffle => CliCommand::QueueShuffle,
            QueueCmd::Show => CliCommand::QueueShow,
        },
        Commands::Library { action } => match action {
            LibraryCmd::Rescan => CliCommand::LibraryRescan,
        },
        Commands::Search {
            scope_or_query,
            query,
        } => {
            if let Some(q) = query {
                let kind = search_kind_from_scope(&scope_or_query)
                    .ok_or_else(|| format!("unknown search scope: {scope_or_query}"))?;
                CliCommand::Search {
                    query: q,
                    kind,
                }
            } else {
                let (q, kind) = parse_search_query(&scope_or_query);

                let kind = if kind == SearchKind::Track
                    && !scope_or_query.contains(':')
                {
                    SearchKind::All
                } else {
                    kind
                };
                CliCommand::Search { query: q, kind }
            }
        }
        Commands::Playlist { action, name } => {
            if let Some(action) = action {
                match action {
                    PlaylistCmd::Create { name } => CliCommand::PlaylistCreate { name },
                    PlaylistCmd::Add { playlist, path } => {
                        let abs = paths::absolutize(&path.to_string_lossy(), cwd);
                        CliCommand::PlaylistAdd {
                            playlist,
                            path: abs.to_string_lossy().into_owned(),
                        }
                    }
                    PlaylistCmd::Remove { playlist, path } => {
                        let abs = paths::absolutize(&path.to_string_lossy(), cwd);
                        CliCommand::PlaylistRemove {
                            playlist,
                            path: abs.to_string_lossy().into_owned(),
                        }
                    }
                    PlaylistCmd::Play { playlist } => CliCommand::PlaylistPlay { playlist },
                    PlaylistCmd::Delete { playlist } => CliCommand::PlaylistDelete { playlist },
                }
            } else if name.is_empty() {
                CliCommand::PlaylistShow { playlist: None }
            } else {
                CliCommand::PlaylistShow {
                    playlist: Some(name.join(" ")),
                }
            }
        }
        Commands::Playlists => CliCommand::ListPlaylists,
        Commands::Albums => CliCommand::ListAlbums,
        Commands::Artists => CliCommand::ListArtists,
        Commands::Album { id_or_name } => CliCommand::ShowAlbum { id_or_name },
        Commands::Artist { id_or_name } => CliCommand::ShowArtist { id_or_name },
        Commands::Open => CliCommand::Open,
        Commands::Hide => CliCommand::Hide,
        Commands::Show => CliCommand::Show,
        Commands::Close => CliCommand::Close,
        Commands::Import { path } => {
            let abs = if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            };
            if !abs.is_dir() {
                return Err(format!("not a directory: {}", abs.display()));
            }
            CliCommand::Import {
                path: abs.to_string_lossy().into_owned(),
            }
        }
        Commands::Info { .. } => unreachable!("handled locally"),
        Commands::Update => CliCommand::Update,
        Commands::Version => unreachable!("handled locally"),
        Commands::Reset { force } => CliCommand::Reset { force },
    })
}

#[cfg(windows)]
fn attach_console() {
    // Attach to parent console so println works with windows_subsystem = "windows"
    type BOOL = i32;
    type DWORD = u32;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AttachConsole(dwProcessId: DWORD) -> BOOL;
        fn AllocConsole() -> BOOL;
    }
    const ATTACH_PARENT_PROCESS: DWORD = 0xFFFFFFFF;
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            let _ = AllocConsole();
        }
    }
}

#[cfg(not(windows))]
fn attach_console() {}


