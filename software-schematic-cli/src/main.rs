use clap::{Parser, Subcommand};
use software_schematic_cli::{Result, init_project, serve};
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
    }
    Ok(())
}
