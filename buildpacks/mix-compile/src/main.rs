use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
use std::process::Command;

#[cfg(test)]
use libcnb_test as _;
use serde::{Deserialize, Serialize};
pub(crate) struct MixCompileBuildpack;

#[derive(Debug)]
enum MixCompileBuildpackError {
    MixCommand(std::io::Error),
}

impl From<MixCompileBuildpackError> for libcnb::Error<MixCompileBuildpackError> {
    fn from(value: MixCompileBuildpackError) -> Self {
        Self::BuildpackError(value)
    }
}

impl Buildpack for MixCompileBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = MixCompileBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        DetectResultBuilder::pass().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        println!("Installing dependencies with `mix deps.get`");
        Command::new("mix")
            .arg("deps.get")
            .status()
            .map_err(MixCompileBuildpackError::MixCommand)?;

        println!("Compiling project with `mix compile`");
        Command::new("mix")
            .arg("compile")
            .status()
            .map_err(MixCompileBuildpackError::MixCommand)?;

        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new("web".parse().unwrap(), ["bash", "-c"])
                            .args(["mix run --no-halt"])
                            .default(true)
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct MixCompileMetadata {
    version: String,
}

buildpack_main!(MixCompileBuildpack);
