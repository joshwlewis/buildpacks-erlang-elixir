// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{
    assert_contains, assert_not_contains, BuildConfig, BuildpackReference, ContainerConfig,
    TestRunner,
};
use std::{env::consts, time::Duration};

const DEFAULT_BUILDER: &str = "heroku/builder:24";

struct IntegrationTestConfig {
    target: String,
    builder: String,
    fixture: String,
}

impl IntegrationTestConfig {
    fn new<S: Into<String>>(fixture: S) -> Self {
        let builder =
            std::env::var("INTEGRATION_TEST_BUILDER").unwrap_or(DEFAULT_BUILDER.to_string());
        let target = match (builder.as_str(), consts::ARCH) {
            // Compile the buildpack for arm64 if the builder supports multi-arch and the host is ARM64.
            // This happens in CI and on developer machines with Apple silicon.
            ("heroku/builder:24", "aarch64") => "aarch64-unknown-linux-musl".to_string(),
            // Compile the buildpack for arm64 if an arm64-specific builder is chosen.
            // Used to run cross-arch integration tests from machines with Intel silicon.
            (b, _) if b.ends_with("arm64") => "aarch64-unknown-linux-musl".to_string(),
            (_, _) => "x86_64-unknown-linux-musl".to_string(),
        };
        let fixture = format!("tests/fixtures/{}", fixture.into());
        Self {
            target,
            builder,
            fixture,
        }
    }
}

impl From<IntegrationTestConfig> for BuildConfig {
    fn from(integration_test_config: IntegrationTestConfig) -> BuildConfig {
        let mut build_config = BuildConfig::new(
            integration_test_config.builder,
            integration_test_config.fixture,
        );
        build_config.buildpacks([
            BuildpackReference::CurrentCrate,
            BuildpackReference::Other(String::from("heroku/procfile")),
        ]);
        build_config.target_triple(integration_test_config.target);
        build_config
    }
}

fn test_erlang_fixture(fixture: &str, expect_loglines: &[&str], refute_loglines: &[&str]) {
    TestRunner::default().build(&IntegrationTestConfig::new(fixture).into(), |ctx| {
        let logs = format!("{}\n{}", ctx.pack_stdout, ctx.pack_stderr);
        for expect_line in expect_loglines {
            assert_contains!(logs, expect_line);
        }
        for refute_line in refute_loglines {
            assert_not_contains!(logs, refute_line);
        }

        let port = 8080;
        ctx.start_container(
            ContainerConfig::new()
                .env("PORT", port.to_string())
                .expose_port(port),
            |container| {
                std::thread::sleep(Duration::from_secs(5));
                let addr = container.address_for_port(port);
                let resp = ureq::get(&format!("http://{addr}"))
                    .call()
                    .expect("request to container failed")
                    .into_string()
                    .expect("response read error");

                assert_contains!(resp, fixture);
            },
        );
    });
}

#[test]
#[ignore = "integration test"]
fn test_simple_erlang_server() {
    test_erlang_fixture("escript_server", &["Installing Erlang/OTP"], &[]);
}
