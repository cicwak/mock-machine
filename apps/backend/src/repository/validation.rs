use crate::{
    domain::{
        ConvertUnknownRequest, CreateProject, CreateScenario, ProfileKind, UpdateProjectSettings,
        UpsertRoute, is_valid_http_method,
    },
    repository::{RepositoryError, RepositoryResult},
};

pub fn validate_convert_request(request: &ConvertUnknownRequest) -> RepositoryResult<()> {
    validate_profile_request(&request.scenario)?;
    for scenario in &request.additional_scenarios {
        validate_profile_request(scenario)?;
    }
    Ok(())
}

pub fn validate_project_request(request: &CreateProject) -> RepositoryResult<()> {
    if request.name.trim().is_empty() {
        return Err(RepositoryError::Validation(
            "project.name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_project_settings(request: &UpdateProjectSettings) -> RepositoryResult<()> {
    if let Some(default_proxy_url) = request.default_proxy_url.as_deref()
        && !(default_proxy_url.starts_with("http://") || default_proxy_url.starts_with("https://"))
    {
        return Err(RepositoryError::Validation(
            "project.default_proxy_url must be an http(s) URL".to_string(),
        ));
    }

    Ok(())
}

pub fn normalize_project_key(key: &str) -> RepositoryResult<String> {
    let key = key.trim().to_ascii_lowercase();
    if is_valid_project_key(&key) {
        Ok(key)
    } else {
        Err(RepositoryError::Validation(
            "project key must be 3-32 chars and contain only lowercase letters, numbers, and hyphens"
                .to_string(),
        ))
    }
}

pub fn validate_route_request(request: &UpsertRoute) -> RepositoryResult<()> {
    if !is_valid_http_method(&request.method) {
        return Err(RepositoryError::Validation(
            "route.method must be a valid HTTP method token".to_string(),
        ));
    }
    if !request.path_pattern.starts_with('/') {
        return Err(RepositoryError::Validation(
            "route.path_pattern must start with /".to_string(),
        ));
    }
    if request.path_pattern == "/mockadmin"
        || request.path_pattern.starts_with("/mockadmin/")
        || request.path_pattern == "/mockadminapi"
        || request.path_pattern.starts_with("/mockadminapi/")
    {
        return Err(RepositoryError::Validation(
            "admin paths cannot be used as mock routes".to_string(),
        ));
    }
    if request.name.trim().is_empty() {
        return Err(RepositoryError::Validation(
            "route.name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_profile_request(request: &CreateScenario) -> RepositoryResult<()> {
    if request.profile_kind == ProfileKind::Dynamic {
        let proxy_url = request.proxy_url.as_deref().unwrap_or_default();
        if !(proxy_url.starts_with("http://") || proxy_url.starts_with("https://")) {
            return Err(RepositoryError::Validation(
                "profile.proxy_url must be an http(s) URL for dynamic profiles".to_string(),
            ));
        }
    }

    if !(100..=599).contains(&request.status_code) {
        return Err(RepositoryError::Validation(
            "scenario.status_code must be between 100 and 599".to_string(),
        ));
    }

    if request.delay_ms < 0 {
        return Err(RepositoryError::Validation(
            "scenario.delay_ms must be non-negative".to_string(),
        ));
    }

    if !request.response_headers.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.response_headers must be a JSON object".to_string(),
        ));
    }

    if !request.selection_rules.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.selection_rules must be a JSON object".to_string(),
        ));
    }

    Ok(())
}

fn is_valid_project_key(key: &str) -> bool {
    (3..=32).contains(&key.len())
        && key
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
        && key
            .bytes()
            .next()
            .is_some_and(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9'))
}
