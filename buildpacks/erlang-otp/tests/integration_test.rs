use buildpack_test_support::test_fixture;
use libcnb_test::BuildpackReference;

#[test]
#[ignore = "integration test"]
fn test_simple_erlang_server() {
    test_fixture(
        "escript-server",
        &[
            BuildpackReference::CurrentCrate,
            BuildpackReference::Other("heroku/procfile".to_string()),
        ],
        &["Installing Erlang/OTP"],
        &[],
    );
}
