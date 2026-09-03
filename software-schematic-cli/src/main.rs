use clap::{Parser, Subcommand};
use software_schematic_cli::{
    Result, assistant_auth_login, assistant_auth_logout, assistant_auth_status, init_project,
    schematic_mcp::serve_mcp, serve, update_project,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ss", version, about = "Software Schematic project wrapper")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install a versioned Software Schematic wrapper in a project.
    Init {
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    #[command(hide = true)]
    Serve {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[arg(long)]
        no_open: bool,
    },
    /// Configure a locally installed Codex or Claude assistant.
    Auth {
        #[arg(long, default_value = ".")]
        project: PathBuf,
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Serve the compiled project schematic over MCP stdio.
    Mcp {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Refresh the pinned runtime and project-local Codex integration.
    Update {
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Open the provider's official account login flow.
    Login {
        /// Prefer codex or claude; Codex is chosen first when omitted.
        #[arg(long)]
        provider: Option<String>,
    },
    /// Show the selected provider and its authentication status.
    Status,
    /// Sign out of the selected provider and clear the project selection.
    Logout,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Init { project } => {
            let layout = init_project(&project)?;
            println!(
                "Software Schematic {} initialized in {}",
                env!("CARGO_PKG_VERSION"),
                layout.project.display()
            );
            println!("Run ./ssw (macOS) or ssw.cmd (Windows) to open it.");
        }
        Command::Serve { project, no_open } => serve(project, !no_open).await?,
        Command::Auth { project, command } => match command {
            AuthCommand::Login { provider } => assistant_auth_login(project, provider.as_deref())?,
            AuthCommand::Status => assistant_auth_status(project)?,
            AuthCommand::Logout => assistant_auth_logout(project)?,
        },
        Command::Mcp { project } => serve_mcp(project).await?,
        Command::Update { project } => {
            let layout = update_project(project)?;
            println!(
                "Software Schematic {} updated in {}",
                env!("CARGO_PKG_VERSION"),
                layout.project.display()
            );
        }
    }
    Ok(())
}
