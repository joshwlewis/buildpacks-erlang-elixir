use buildpack_test_support::test_fixture;
use libcnb_test::BuildpackReference;

#[test]
#[ignore = "integration test"]
fn test_plug_server() {
    test_fixture(
        "plug-server",
        &[
            BuildpackReference::WorkspaceBuildpack("joshwlewis/erlang-otp".parse().unwrap()),
            BuildpackReference::WorkspaceBuildpack("joshwlewis/elixir-engine".parse().unwrap()),
            BuildpackReference::CurrentCrate,
        ],
        &["mix deps.get", "mix compile"],
        &[],
    );
}
