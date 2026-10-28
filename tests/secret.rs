use ax_exchange_sdk::{protocol::api_gateway::AuthenticationMethod, types::Secret};

#[test]
fn a_secret_is_redacted_from_debug() {
    let auth = AuthenticationMethod::ApiKeySecret {
        api_key: "key".to_string(),
        api_secret: Secret::from("hunter2"),
    };
    assert!(!format!("{auth:?}").contains("hunter2"));
}

#[test]
fn a_secret_goes_over_the_wire_as_a_plain_string() {
    let auth = AuthenticationMethod::ApiKeySecret {
        api_key: "key".to_string(),
        api_secret: Secret::from("hunter2"),
    };
    let json = serde_json::to_value(&auth).unwrap();
    assert_eq!(json["api_secret"], "hunter2");
    let back: AuthenticationMethod = serde_json::from_value(json).unwrap();
    assert_eq!(back, auth);
}
