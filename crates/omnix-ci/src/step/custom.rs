//! Custom steps in the CI pipeline
use colored::Colorize;
use nonempty::NonEmpty;
use serde::Deserialize;
use std::{collections::BTreeMap, future::Future, path::PathBuf};

use nix_rs::{
    command::NixCmd,
    flake::{
        self,
        system::System,
        url::{attr::FlakeAttr, FlakeUrl},
    },
};

use crate::config::subflake::SubflakeConfig;

/// Represents a custom step in the CI pipeline
///
/// All these commands are run in the same directory as the subflake
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum CustomStep {
    /// A flake app to run
    #[serde(rename = "app")]
    FlakeApp {
        /// Name of the app
        #[serde(default)]
        name: FlakeAttr,
        /// Arguments to pass to the app
        #[serde(default)]
        args: Vec<String>,
        /// Whitelist of systems to run on
        systems: Option<Vec<System>>,
    },

    /// An arbitrary command to run in the devshell
    #[serde(rename = "devshell")]
    FlakeDevShellCommand {
        /// Name of the devShell
        #[serde(default)]
        name: FlakeAttr,
        /// The command to run inside of devshell
        command: NonEmpty<String>,
        /// Whitelist of systems to run on
        systems: Option<Vec<System>>,
    },
}

impl CustomStep {
    /// Run this step
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        with_writeable_flake_dir(nixcmd, url, |flake_path| async move {
            self.run_on_local_path(nixcmd, flake_path, subflake).await
        })
        .await
    }

    /// Like [run] but runs on a flake that is known to be at a local path
    async fn run_on_local_path(
        &self,
        nixcmd: &NixCmd,
        flake_path: PathBuf,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        let path = flake_path.join(&subflake.dir);
        tracing::info!("Running custom step under: {:}", &path.display());

        let pwd_flake = FlakeUrl::from(PathBuf::from("."));
        let flake_opts = flake::command::FlakeOptions {
            override_inputs: subflake.override_inputs.clone(),
            current_dir: Some(path.clone()),
            no_write_lock_file: false,
        };

        match self {
            CustomStep::FlakeApp { name, args, .. } => {
                flake::command::run(
                    nixcmd,
                    &flake_opts,
                    &pwd_flake.with_attr(&name.get_name()),
                    args.clone(),
                )
                .await?;
            }
            CustomStep::FlakeDevShellCommand { name, command, .. } => {
                flake::command::develop(
                    nixcmd,
                    &flake_opts,
                    &pwd_flake.with_attr(&name.get_name()),
                    command.clone(),
                )
                .await?;
            }
        }
        Ok(())
    }

    fn can_run_on(&self, systems: &[System]) -> bool {
        match self.get_systems() {
            Some(systems_whitelist) => systems_whitelist.iter().any(|s| systems.contains(s)),
            None => true,
        }
    }

    fn get_systems(&self) -> &Option<Vec<System>> {
        match self {
            CustomStep::FlakeApp { systems, .. } => systems,
            CustomStep::FlakeDevShellCommand { systems, .. } => systems,
        }
    }
}

/// A collection of custom steps
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CustomSteps(BTreeMap<String, CustomStep>);

impl CustomSteps {
    /// Run all custom steps
    pub async fn run(
        &self,
        nixcmd: &NixCmd,
        systems: &[System],
        url: &FlakeUrl,
        subflake: &SubflakeConfig,
    ) -> anyhow::Result<()> {
        for (name, step) in &self.0 {
            if step.can_run_on(systems) {
                tracing::info!("{}", format!("🏗  Running custom step: {}", name).bold());
                step.run(nixcmd, url, subflake).await?;
            } else {
                tracing::info!(
                  "{}",
                  format!(
                      "🏗  Skipping custom step {} because it's not whitelisted for the current system: {:?}",
                      name,
                      systems.iter().map(|s| s.to_string()).collect::<Vec<_>>()
                  )
                  .yellow()
              );
            }
        }
        Ok(())
    }
}

/// Call the given function with a (write-able) local path equivalent to the given URL
///
/// The flake is retrieved locally, and stored in a temp directory is created if necessary.
///
/// Two reasons for copying to a temp (and writeable) directory:
/// 1. `nix run` does not work reliably on store paths (`/nix/store/**`)
/// 2. `nix develop -c ...` often requires mutable flake directories
async fn with_writeable_flake_dir<F, Fut>(
    nixcmd: &NixCmd,
    url: &FlakeUrl,
    f: F,
) -> anyhow::Result<()>
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    // First, ensure that flake is locally available.
    let local_path = match url.as_local_path() {
        Some(local_path) => local_path.to_path_buf(),
        None => url.as_local_path_or_fetch(nixcmd).await?,
    };

    // Then, ensure that it is writeable by the user
    let read_only = local_path.metadata()?.permissions().readonly();
    let path = if read_only {
        // Two reasons for copying to a temp location:
        // 1. `nix run` does not work reliably on store paths
        // 2. `nix develop -c ...` often require mutable flake directories
        let target_path = tempfile::Builder::new()
            .prefix("om-ci-")
            .tempdir()?
            .path()
            .join("flake");
        omnix_common::fs::copy_dir_all(&local_path, &target_path).await?;
        target_path
    } else {
        local_path
    };

    // Finally, call the function with the path
    f(path).await
}
