lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub broker_url: String,
    pub broker_user: String,
    pub broker_pass: String,
    pub otlp_collector: Option<String>,
    pub otlp_auth: Option<String>,
    pub rust_log: Option<String>,
    pub image_version: String,
}

pub fn get(env_variable: &str, default: &str) -> String {
    std::env::var(env_variable).unwrap_or(default.to_string())
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            broker_url: get("BROKER_URL", "mqtt://localhost:1883"),
            broker_user: get("BROKER_USER", "mqtt"),
            broker_pass: get("BROKER_PASS", "pass"),
            otlp_collector: std::env::var("OTLP_COLLECTOR").ok(),
            otlp_auth: std::env::var("OLTP_AUTH").ok(),
            rust_log: std::env::var("RUST_LOG").ok(),
            image_version: get("RUMSIM_VERSION", "latest"),
        }
    }
}
