use std::process::Command;

use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};

// Suppress warnings due to the `unused_crate_dependencies` lint not handling integration tests well.
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
#[cfg(test)]
use libcnb_test as _;
use serde::{Deserialize, Serialize};

mod bob;
mod tgz;

pub(crate) struct ErlangOTPBuildpack;

#[derive(Debug)]
pub(crate) enum ErlangOTPBuildpackError {
    ListVersions(bob::Error),
    ResolveVersion,
    DownloadBuild(tgz::Error),
    TempDir(std::io::Error),
    Install(std::io::Error),
}

impl From<ErlangOTPBuildpackError> for libcnb::Error<ErlangOTPBuildpackError> {
    fn from(value: ErlangOTPBuildpackError) -> Self {
        Self::BuildpackError(value)
    }
}

impl Buildpack for ErlangOTPBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = ErlangOTPBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        DetectResultBuilder::pass().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        // TODO: Allow user to pick Erlang/OTP version.
        let otp_version = "26.2.4";
        let metadata = ErlangOTPMetadata {
            version: otp_version.to_string(),
        };

        let dist_layer = context.cached_layer(
            layer_name!("dist"),
            CachedLayerDefinition {
                build: true,
                launch: true,
                invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
                restored_layer_action: &|previous_metadata: &ErlangOTPMetadata, _| {
                    if previous_metadata == &metadata {
                        RestoredLayerAction::KeepLayer
                    } else {
                        RestoredLayerAction::DeleteLayer
                    }
                },
            },
        )?;

        match dist_layer.state {
            LayerState::Restored { .. } => {
                println!("Restoring Erlang/OTP {otp_version} from cache");
            }
            LayerState::Empty { .. } => {
                println!("Resolving Erlang/OTP version (requested {otp_version})");

                let erlang_builds = bob::ErlangBuild::list(
                    context.target.arch,
                    context.target.distro_name,
                    context.target.distro_version,
                )
                .map_err(ErlangOTPBuildpackError::ListVersions)?;

                // TODO: Use semver logic to resolve selected build.
                let erlang_build = erlang_builds
                    .iter()
                    .find(|build| build.version.contains(otp_version))
                    .ok_or(ErlangOTPBuildpackError::ResolveVersion)?;

                println!("Downloading Erlang/OTP from {}", erlang_build.url());
                tgz::fetch_extract_strip(erlang_build.url(), dist_layer.path())
                    .map_err(ErlangOTPBuildpackError::DownloadBuild)?;

                println!("Installing Erlang/OTP version {}", erlang_build.version);
                Command::new(dist_layer.path().join("Install"))
                    .arg("-minimal")
                    .arg(dist_layer.path())
                    .status()
                    .map_err(ErlangOTPBuildpackError::Install)?;
            }
        }
        dist_layer.write_metadata(metadata)?;

        BuildResultBuilder::new().build()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct ErlangOTPMetadata {
    version: String,
}

buildpack_main!(ErlangOTPBuildpack);
