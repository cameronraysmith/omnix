use clap::Parser;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_health::{run_all_checks_with, NixHealth};

/// Display the health of your Nix dev environment
#[derive(Parser, Debug)]
pub struct HealthCommand {
    /// Use `om.health` configuration from the given flake
    #[arg(name = "FLAKE")]
    pub flake_url: Option<FlakeUrl>,

    /// Dump the config schema of the health checks (useful when adding them to
    /// a flake.nix)
    #[arg(long = "dump-schema")]
    pub dump_schema: bool,

    /// Print output in JSON
    #[arg(long)]
    json: bool,

    /// Nix command global options
    #[command(flatten)]
    pub nixcmd: NixCmd,
}

impl HealthCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        if self.dump_schema {
            println!("{}", NixHealth::schema()?);
        } else {
            let checks =
                run_all_checks_with(&self.nixcmd, self.flake_url.clone(), self.json).await?;
            let exit_code = NixHealth::print_report_returning_exit_code(&checks, self.json).await?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Ok(())
    }
}
