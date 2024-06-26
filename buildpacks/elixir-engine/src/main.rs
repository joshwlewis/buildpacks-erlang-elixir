use std::fs::create_dir_all;
use std::process::Command;

use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::{buildpack_main, Buildpack, Env};

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
    CreateDir(std::io::Error),
    MixCommand(std::io::Error),
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
        let mut build_env = Env::from_current();
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
        dist_layer.write_metadata(&metadata)?;
        build_env = dist_layer.read_env()?.apply(Scope::Build, &build_env);
        let mix_layer = context.cached_layer(
            layer_name!("mix"),
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
        let mix_home_dir = mix_layer.path().join("home");
        let mix_archives_dir = mix_layer.path().join("archives");
        let mix_build_dir = mix_layer.path().join("build");
        // TODO: mix_deps_dir cache should expire seperately from elixir version.
        let mix_deps_dir = mix_layer.path().join("deps");
        let mix_install_dir = mix_layer.path().join("install");
        match mix_layer.state {
            LayerState::Restored { .. } => {
                println!("Restoring Mix directories.")
            }
            LayerState::Empty { .. } => {
                println!("Creating Mix directories.");
                for dir in [
                    &mix_home_dir,
                    &mix_archives_dir,
                    &mix_build_dir,
                    &mix_deps_dir,
                    &mix_install_dir,
                ] {
                    create_dir_all(dir).map_err(ElixirEngineBuildpackError::CreateDir)?;
                }
                mix_layer.write_env(
                    LayerEnv::new()
                        .chainable_insert(
                            Scope::All,
                            ModificationBehavior::Override,
                            "MIX_HOME",
                            mix_home_dir,
                        )
                        .chainable_insert(
                            Scope::All,
                            ModificationBehavior::Override,
                            "MIX_ARCHIVES",
                            mix_archives_dir,
                        )
                        .chainable_insert(
                            Scope::All,
                            ModificationBehavior::Override,
                            "MIX_BUILD_ROOT",
                            mix_build_dir,
                        )
                        .chainable_insert(
                            libcnb::layer_env::Scope::All,
                            ModificationBehavior::Override,
                            "MIX_DEPS_PATH",
                            mix_deps_dir,
                        )
                        .chainable_insert(
                            libcnb::layer_env::Scope::All,
                            ModificationBehavior::Override,
                            "MIX_INSTALL_DIR",
                            mix_install_dir,
                        ),
                )?;
            }
        };
        mix_layer.write_metadata(&metadata)?;
        build_env = mix_layer.read_env()?.apply(Scope::Build, &build_env);

        Command::new("mix")
            .arg("local.hex")
            .arg("--force")
            .envs(&build_env)
            .status()
            .map_err(ElixirEngineBuildpackError::MixCommand)?;

        Command::new("mix")
            .arg("local.rebar")
            .arg("--force")
            .envs(&build_env)
            .status()
            .map_err(ElixirEngineBuildpackError::MixCommand)?;

        BuildResultBuilder::new().build()
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct ElixirEngineMetadata {
    version: String,
}

buildpack_main!(ElixirEngineBuildpack);
