#[derive(Debug, Clone)]
pub struct Settings {
    pub broker_url: String,
    pub broker_user: String,
    pub broker_pass: String,
    pub otlp_collector: Option<String>,
    pub otlp_auth: Option<String>,
}

pub fn get(env_variable: &str, default: &str) -> String {
    std::env::var(env_variable).unwrap_or(default.to_string())
}

pub fn get_num(env_variable: &str, default: usize) -> usize {
    std::env::var(env_variable)
        .unwrap_or(default.to_string())
        .parse()
        .unwrap() // It's OK to panic if someone sets a broken number in the environment.
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            broker_url: get("URL", "mqtt://localhost:1883"),
            broker_user: get("USER", "mqtt"),
            broker_pass: get("PASS", "pass"),
            otlp_collector: std::env::var("OTLP_ENDPOINT").ok(),
            otlp_auth: std::env::var("OLTP_AUTH").ok(),
        }
    }
}
