use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddAuthArgs {
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default, rename = "sessionFields")]
    pub session_fields: Vec<String>,
    #[serde(default, rename = "emailVerification")]
    pub email_verification: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddAuthGuardArgs {
    #[serde(rename = "routePath")]
    pub route_path: String,
    #[serde(default = "default_redirect", rename = "redirectTo")]
    pub redirect_to: String,
}

fn default_redirect() -> String {
    "/login".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_auth_args_deserialize() {
        let json = r#"{
            "providers": ["github", "google"],
            "sessionFields": ["role"],
            "emailVerification": true
        }"#;
        let args: AddAuthArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.providers.len(), 2);
        assert!(args.email_verification);
    }

    #[test]
    fn test_add_auth_guard_args_deserialize() {
        let json = r#"{
            "routePath": "/admin",
            "redirectTo": "/login"
        }"#;
        let args: AddAuthGuardArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.route_path, "/admin");
        assert_eq!(args.redirect_to, "/login");
    }
}
