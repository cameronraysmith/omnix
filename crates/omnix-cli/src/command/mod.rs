use clap::Subcommand;

pub mod ci;
pub mod init;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Inspect a flake
    Show(show::ShowConfig),

    /// Initialize a flake
    Init(init::InitConfig),

    /// Build all flake outputs (run CI locally)
    CI(ci::CIConfig),
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Show(config) => config.run().await,
            Command::Init(config) => config.run().await,
            Command::CI(config) => config.run().await,
        }
    }
}
