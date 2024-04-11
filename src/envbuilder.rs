use k8s_openapi::api::core::v1::{EnvVar, EnvVarSource, SecretKeySelector};

pub struct EnvVarBuilder {
    env: Vec<EnvVar>,
}

impl EnvVarBuilder {
    pub fn new() -> Self {
        EnvVarBuilder { env: Vec::new() }
    }

    pub fn add_u64(&mut self, name: &str, value: u64) -> &mut Self {
        let var = EnvVar {
            name: name.to_string(),
            value: Some(value.to_string()),
            ..Default::default()
        };
        self.env.push(var);
        self
    }

    pub fn add_u64_opt(&mut self, name: &str, value: Option<u64>) -> &mut Self {
        if let Some(v) = value {
            self.add_u64(name, v);
        }
        self
    }

    pub fn add_str(&mut self, name: &str, value: &str) -> &mut Self {
        let var = EnvVar {
            name: name.to_string(),
            value: Some(value.to_string()),
            ..Default::default()
        };
        self.env.push(var);
        self
    }

    pub fn add_str_opt(&mut self, name: &str, value: Option<String>) -> &mut Self {
        if let Some(v) = value {
            self.add_str(name, &v);
        }
        self
    }

    pub fn add_secret_ref(&mut self, name: &str, secret: &str, key: &str) -> &mut Self {
        let var = EnvVar {
            name: name.to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: Some(secret.to_string()),
                    key: key.to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.env.push(var);
        self
    }

    pub fn add_field_ref(&mut self, name: &str, field: &str) -> &mut Self {
        let var = EnvVar {
            name: name.to_string(),
            value_from: Some(EnvVarSource {
                field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                    field_path: field.to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        self.env.push(var);
        self
    }

    pub fn build(&self) -> Vec<EnvVar> {
        self.env.clone()
    }
}
