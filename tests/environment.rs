use ax_exchange_sdk::{ArchitectX, environment::Environment};

#[test]
fn a_malformed_custom_url_is_an_error_not_a_panic() {
    let env = Environment::Custom("not a url".to_string());
    assert!(env.base_url().is_err());
    assert!(ArchitectX::new(env, None::<&str>, None::<&str>).is_err());
}

#[test]
fn the_named_environments_parse() {
    assert!(Environment::Production.base_url().is_ok());
    assert!(Environment::Sandbox.base_url().is_ok());
}
