use buildpack_test_support::test_fixture;
use libcnb_test::BuildpackReference;

#[test]
#[ignore = "integration test"]
fn test_simple_erlang_server() {
    test_fixture(
        "exs-server",
        &[
            BuildpackReference::WorkspaceBuildpack("joshwlewis/erlang-otp".parse().unwrap()),
            BuildpackReference::CurrentCrate,
            BuildpackReference::Other("heroku/procfile".to_string()),
        ],
        &["Downloading Elixir"],
        &[],
    );
}
