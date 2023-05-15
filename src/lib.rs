pub struct EnvyToml;

impl EnvyToml {
    pub fn from_str<T>(s: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let toml = toml::from_str::<toml::Value>(s)?;
        let table = toml.as_table().ok_or("Extracts the table value error")?;
        Self::set_env_from_table(None, table);
        Ok(envy::from_env::<T>()?)
    }

    fn set_env_from_table(prefix: Option<String>, table: &toml::Table) {
        table.iter().for_each(|(k, v)| {
            let key = prefix.clone().map_or(k.clone(), |p| format!("{p}_{k}"));
            match v.clone() {
                toml::Value::String(s) => Self::set_env_from_string(&key, &s),
                toml::Value::Integer(i) => Self::set_env_from_string(&key, &i.to_string()),
                toml::Value::Table(t) => Self::set_env_from_table(Some(key), &t),
                value => panic!("Unimplemented handler for value type: {:?}", value),
            };
        })
    }

    fn set_env_from_string(key: &str, v_toml: &str) {
        let k1 = key.to_lowercase();
        let k2 = key.to_uppercase();
        match (std::env::var(&k1), std::env::var(&k2)) {
            (Ok(_v1), Ok(_v2)) => {
                std::env::remove_var(&k2);
                #[cfg(debug_assertions)]
                println!("\x1B[1mEnvironment var `{k1}` overridden with `{_v1}` (unused `{k2}`=`{_v2}`)\x1B[0m")
            }
            (Ok(_v), Err(_)) => {
                #[cfg(debug_assertions)]
                println!("\x1B[1mEnvironment var `{k1}` overridden with `{_v} (in config.toml `{v_toml}`)`\x1B[0m",)
            }
            (Err(_), Ok(_v)) => {
                #[cfg(debug_assertions)]
                println!("\x1B[1mEnvironment var `{k2}` overridden with `{_v} (in config.toml `{v_toml}`)`\x1B[0m",)
            }
            (Err(_), Err(_)) => std::env::set_var(key.to_uppercase(), v_toml),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn it_works() {
        std::env::set_var("overridden_both", "value_from_both_env");
        std::env::set_var("OVERRIDDEN_BOTH", "value_from_both_env");
        std::env::set_var("overridden_lower", "value_from_lower_env");
        std::env::set_var("OVERRIDDEN_UPPER", "value_from_upper_env");

        #[derive(Deserialize)]
        struct Config {
            example_key: String,
            overridden_both: String,
            overridden_lower: String,
            overridden_upper: String,
        }

        let config = EnvyToml::from_str::<Config>(
            r#"
            example_key = "example_key_from_config"

            [overridden]
            both  = "both_from_config"
            lower = "lower_from_config"
            upper = "upper_from_config"
            "#,
        )
        .unwrap();

        assert_eq!(config.example_key, "example_key_from_config");
        // Should be overridden with env var in lowercase when input both cases
        assert_eq!(config.overridden_both, "value_from_both_env");
        // Should be overridden with env var in lowercase
        assert_eq!(config.overridden_lower, "value_from_lower_env");
        // Should be overridden with env var in uppercase
        assert_eq!(config.overridden_upper, "value_from_upper_env");
    }
}
