use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct BuildLogEntry {
    message: String,
    severity: String,
    timestamp: String,
}

fn strip_ansi_codes(text: &str) -> String {
    // Simple regex to remove ANSI escape sequences
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(text, "").to_string()
}

fn should_stop_at_message(message: &str) -> bool {
    let clean_message = strip_ansi_codes(message);

    // Check for "Build time: X seconds" pattern (case insensitive)
    let build_time_pattern = regex::Regex::new(r"(?i)Build\s+time:\s+\d+").unwrap();
    if build_time_pattern.is_match(&clean_message) {
        return true;
    }

    // Check for "Starting Container" (case insensitive)
    let starting_container_pattern = regex::Regex::new(r"(?i)Starting\s+Container").unwrap();
    if starting_container_pattern.is_match(&clean_message) {
        return true;
    }

    false
}

pub async fn fetch_build_logs() -> Result<crate::models::BuildLogs> {
    let token = env::var("RAILWAY_TOKEN")?;
    let deployment_id = env::var("RAILWAY_DEPLOYMENT_ID")?;

    let query = r#"
        query buildLogs($deploymentId: String!, $endDate: DateTime, $filter: String, $limit: Int, $startDate: DateTime) {
            buildLogs(
                deploymentId: $deploymentId
                endDate: $endDate
                filter: $filter
                limit: $limit
                startDate: $startDate
            ) {
                message
                severity
                timestamp
            }
        }
    "#;

    let variables = serde_json::json!({
        "deploymentId": deployment_id,
        "limit": 1000
    });

    let request = GraphQLRequest {
        query: query.to_string(),
        variables,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://backboard.railway.app/graphql/v2")
        .header("Authorization", format!("Bearer {}", token))
        .json(&request)
        .send()
        .await?;

    let response_text = response.text().await?;
    let graphql_response: GraphQLResponse = serde_json::from_str(&response_text)?;

    if let Some(errors) = graphql_response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(anyhow::anyhow!(
            "GraphQL errors: {}",
            error_messages.join(", ")
        ));
    }

    if let Some(data) = graphql_response.data {
        if let Some(build_logs_value) = data.get("buildLogs") {
            if let Ok(build_logs) =
                serde_json::from_value::<Vec<BuildLogEntry>>(build_logs_value.clone())
            {
                let mut filtered_logs = Vec::new();

                for entry in build_logs {
                    // Check if we should stop at this message
                    if should_stop_at_message(&entry.message) {
                        // For "Build time" messages, include them
                        // For "Starting Container" messages, stop before them
                        let clean_message = strip_ansi_codes(&entry.message);
                        let starting_container_pattern =
                            regex::Regex::new(r"(?i)Starting\s+Container").unwrap();

                        if starting_container_pattern.is_match(&clean_message) {
                            // Stop before "Starting Container" message
                            break;
                        } else {
                            // Include "Build time" message and stop
                            let formatted_entry = format!(
                                "{} {} {}",
                                entry.timestamp,
                                entry.severity,
                                clean_message.trim()
                            );
                            filtered_logs.push(formatted_entry);
                            break;
                        }
                    }

                    // Include this log entry
                    let clean_message = strip_ansi_codes(&entry.message);
                    let formatted_entry = format!(
                        "{} {} {}",
                        entry.timestamp,
                        entry.severity,
                        clean_message.trim()
                    );
                    filtered_logs.push(formatted_entry);
                }

                                // Add Railway URL header to the logs
                let railway_url = format!(
                    "Railway Build Logs: https://railway.com/project/{}/service/{}?environmentId={}&id={}#build\n\n",
                    env::var("RAILWAY_PROJECT_ID").unwrap_or_default(),
                    env::var("RAILWAY_SERVICE_ID").unwrap_or_default(),
                    env::var("RAILWAY_ENVIRONMENT_ID").unwrap_or_default(),
                    deployment_id
                );
                
                let content = format!("{}{}", railway_url, filtered_logs.join("\n"));
                let fetched_at = chrono::Utc::now();
                
                // Generate hash for the content
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let content_hash = hasher.finish();

                return Ok(crate::models::BuildLogs {
                    content,
                    fetched_at,
                    content_hash,
                });
            }
        }

        Err(anyhow::anyhow!(
            "Unexpected response structure from Railway API"
        ))
    } else {
        Err(anyhow::anyhow!("No data received from Railway API"))
    }
}
