use std::{collections::HashMap, env, time::SystemTime};

use crate::app_error::AppError;

type EnvHashMap = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct AppEnv {
    pub log_level: tracing::Level,
    pub start_time: SystemTime,
    pub ws_address: String,
    pub ws_apikey: String,
    pub ws_password: String,
    pub ws_token_address: String,
}

impl AppEnv {
    /// Parse "true" or "false" to bool, else false
    fn parse_boolean(key: &str, map: &EnvHashMap) -> bool {
        map.get(key).map_or(false, |value| value == "true")
    }

    fn parse_string(key: &str, map: &EnvHashMap) -> Result<String, AppError> {
        map.get(key)
            .map_or(Err(AppError::MissingEnv(key.into())), |value| {
                Ok(value.into())
            })
    }

    /// Parse debug and/or trace into tracing level
    fn parse_log(map: &EnvHashMap) -> tracing::Level {
        if Self::parse_boolean("LOG_TRACE", map) {
            tracing::Level::TRACE
        } else if Self::parse_boolean("LOG_DEBUG", map) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Load, and parse .env file, return `AppEnv`
    fn generate() -> Result<Self, AppError> {
        let env_map = env::vars()
            .map(|i| (i.0, i.1))
            .collect::<HashMap<String, String>>();

        Ok(Self {
            log_level: Self::parse_log(&env_map),
            start_time: SystemTime::now(),
            ws_address: Self::parse_string("WS_ADDRESS", &env_map)?,
            ws_apikey: Self::parse_string("WS_APIKEY", &env_map)?,
            ws_password: Self::parse_string("WS_PASSWORD", &env_map)?,
            ws_token_address: Self::parse_string("WS_TOKEN_ADDRESS", &env_map)?,
        })
    }

    pub fn get() -> Self {
        let local_env = ".env";
        let app_env = "/app_env/.env";

        let env_path = if std::fs::metadata(app_env).is_ok() {
            app_env
        } else if std::fs::metadata(local_env).is_ok() {
            local_env
        } else {
            println!("\n\x1b[31munable to load env file\x1b[0m\n");
            std::process::exit(1);
        };

        dotenvy::from_path(env_path).ok();
        match Self::generate() {
            Ok(s) => s,
            Err(e) => {
                println!("\n\x1b[31m{e}\x1b[0m\n");
                std::process::exit(1);
            }
        }
    }
}

/// Run tests with
///
/// cargo watch -q -c -w src/ -x 'test env_ -- --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn env_missing_env() {
        let mut map = HashMap::new();
        map.insert("not_fish".to_owned(), "not_fish".to_owned());

        let result = AppEnv::parse_string("fish", &map);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "missing env: 'fish'");
    }

    #[tokio::test]
    async fn env_parse_string_valid() {
        let mut map = HashMap::new();
        map.insert("LOCATION_SQLITE".to_owned(), "/alarms.db".to_owned());

        let result = AppEnv::parse_string("LOCATION_SQLITE", &map).unwrap();

        assert_eq!(result, "/alarms.db");
    }

    #[tokio::test]
    async fn env_parse_boolean_ok() {
        let mut map = HashMap::new();
        map.insert("valid_true".to_owned(), "true".to_owned());
        map.insert("valid_false".to_owned(), "false".to_owned());
        map.insert("invalid_but_false".to_owned(), "as".to_owned());

        let result01 = AppEnv::parse_boolean("valid_true", &map);
        let result02 = AppEnv::parse_boolean("valid_false", &map);
        let result03 = AppEnv::parse_boolean("invalid_but_false", &map);
        let result04 = AppEnv::parse_boolean("missing", &map);

        assert!(result01);
        assert!(!result02);
        assert!(!result03);
        assert!(!result04);
    }

    #[test]
    fn env_parse_log_valid() {
        let map = HashMap::from([("RANDOM_STRING".to_owned(), "123".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([("LOG_DEBUG".to_owned(), "false".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([("LOG_TRACE".to_owned(), "false".to_owned())]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::INFO);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "false".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::DEBUG);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "true".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::TRACE);

        let map = HashMap::from([
            ("LOG_DEBUG".to_owned(), "false".to_owned()),
            ("LOG_TRACE".to_owned(), "true".to_owned()),
        ]);

        let result = AppEnv::parse_log(&map);

        assert_eq!(result, tracing::Level::TRACE);
    }

    // Why?
    #[tokio::test]
    async fn env_panic_appenv() {
        let result = AppEnv::generate();

        assert!(result.is_err());

        dotenvy::dotenv().ok();

        let result = AppEnv::generate();

        assert!(result.is_ok());
    }
}