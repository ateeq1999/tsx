use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddSeedArgs {
    pub name: String,
    #[serde(default = "default_count")]
    pub count: usize,
}

fn default_count() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_seed_args_deserialize() {
        let json = r#"{
            "name": "products",
            "count": 5
        }"#;
        let args: AddSeedArgs = serde_json::from_str(json).unwrap();
        assert_eq!(args.name, "products");
        assert_eq!(args.count, 5);
    }
}
