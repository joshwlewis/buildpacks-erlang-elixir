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
mod zip;
pub(crate) struct ElixirEngineBuildpack;

#[derive(Debug)]
pub(crate) enum ElixirEngineBuildpackError {
    ListVersions(bob::Error),
    ResolveVersion,
    DownloadBuild(zip::Error),
}

impl From<ElixirEngineBuildpackError> for libcnb::Error<ElixirEngineBuildpackError> {
    fn from(value: ElixirEngineBuildpackError) -> Self {
        Self::BuildpackError(value)
    }
}

impl Buildpack for ElixirEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = ElixirEngineBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        DetectResultBuilder::pass().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        // TODO: Allow user to pick Elixir version.
        let elixir_version = "1.17.1";
        let metadata = ElixirEngineMetadata {
            version: elixir_version.to_string(),
        };

        let dist_layer = context.cached_layer(
            layer_name!("dist"),
            CachedLayerDefinition {
                build: true,
                launch: true,
                invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
                restored_layer_action: &|previous_metadata: &ElixirEngineMetadata, _| {
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
                println!("Restoring Elixir {elixir_version} from cache");
            }
            LayerState::Empty { .. } => {
                println!("Resolving Elixir version (requested {elixir_version})");

                let elixir_builds =
                    bob::ElixirBuild::list().map_err(ElixirEngineBuildpackError::ListVersions)?;

                // TODO: Use semver logic to resolve selected build.
                let elixir_build = elixir_builds
                    .iter()
                    .find(|build| build.version.contains(elixir_version))
                    .ok_or(ElixirEngineBuildpackError::ResolveVersion)?;

                println!("Downloading Elixir from {}", elixir_build.url());
                zip::fetch_strip_unzip(elixir_build.url(), dist_layer.path())
                    .map_err(ElixirEngineBuildpackError::DownloadBuild)?;
            }
        }
        dist_layer.write_metadata(metadata)?;

        BuildResultBuilder::new().build()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct ElixirEngineMetadata {
    version: String,
}

buildpack_main!(ElixirEngineBuildpack);
