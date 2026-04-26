mod cli;
mod client;
mod config;
mod core;
mod store;
mod ui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "seedance",
    about = "CLI for Seedance 2.0 video generation on Volcengine Ark",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a video generation task (T2V / I2V / V2V / R2V)
    Generate(cli::generate::GenerateArgs),

    /// Extend a video forward/backward, or bridge multiple clips
    Extend(cli::extend::ExtendArgs),

    /// Edit an existing video (replace, add, remove, repaint)
    Edit(cli::edit::EditArgs),

    /// Query task status
    Status(cli::status::StatusArgs),

    /// Download a completed video
    Download(cli::download::DownloadArgs),

    /// List task history
    List(cli::list::ListArgs),

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: cli::config::ConfigCommand,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Generate(args) => cli::generate::execute(args).await,
        Commands::Extend(args) => cli::extend::execute(args).await,
        Commands::Edit(args) => cli::edit::execute(args).await,
        Commands::Status(args) => cli::status::execute(args).await,
        Commands::Download(args) => cli::download::execute(args).await,
        Commands::List(args) => cli::list::execute(args).await,
        Commands::Config { command } => cli::config::execute(command).await,
    };

    if let Err(e) = result {
        ui::print_error(&format!("{e:#}"));
        std::process::exit(1);
    }
}
