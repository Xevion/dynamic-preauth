use serde::Deserialize;

fn default_port() -> u16 {
    5800
}

/// Railway-specific configuration parsed from environment variables.
#[derive(Deserialize, Debug, Default)]
pub struct RailwayConfig {
    pub railway_token: Option<String>,
    pub railway_project_id: Option<String>,
    pub railway_service_id: Option<String>,
    pub railway_environment_id: Option<String>,
    pub railway_deployment_id: Option<String>,
    pub railway_public_domain: Option<String>,
}

impl RailwayConfig {
    /// Returns true if running on Railway (project ID is set).
    pub fn is_railway(&self) -> bool {
        self.railway_project_id.is_some()
    }

    /// Returns true if Railway API token is configured.
    pub fn has_token(&self) -> bool {
        self.railway_token.is_some()
    }

    /// Build the Railway dashboard URL for viewing build logs.
    pub fn build_logs_url(&self) -> Option<String> {
        let project_id = self.railway_project_id.as_ref()?;
        let service_id = self.railway_service_id.as_ref()?;
        let environment_id = self.railway_environment_id.as_ref()?;
        let deployment_id = self
            .railway_deployment_id
            .as_deref()
            .unwrap_or("latest");

        Some(format!(
            "https://railway.com/project/{}/service/{}?environmentId={}&id={}#build",
            project_id, service_id, environment_id, deployment_id
        ))
    }

    /// Returns the CORS origin based on public domain.
    pub fn cors_origin(&self) -> String {
        if cfg!(debug_assertions) {
            return "*".to_string();
        }

        match &self.railway_public_domain {
            Some(domain) => format!("https://{}", domain),
            None => "*".to_string(),
        }
    }
}

/// Main configuration struct parsed from environment variables.
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(flatten)]
    pub railway: RailwayConfig,
}

impl Config {
    /// Returns the socket address to bind to.
    pub fn bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
