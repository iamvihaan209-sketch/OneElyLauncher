use chrono::{Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::data::{AccountKind, MinecraftAccount};
use super::error::{MinecraftAuthError, MinecraftAuthStep};

const AUTH_URL: &str = "https://authserver.ely.by/auth/authenticate";
const REFRESH_URL: &str = "https://authserver.ely.by/auth/refresh";

#[derive(Debug, Deserialize)]
struct Profile {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    selected_profile: Profile,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

fn request_error(step: MinecraftAuthStep, err: reqwest::Error) -> MinecraftAuthError {
    MinecraftAuthError::RequestError { step, source: err }
}

async fn post<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
    body: serde_json::Value,
    step: MinecraftAuthStep,
) -> Result<T, MinecraftAuthError> {
    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| request_error(step, e))?;

    let status = response.status();
    let raw = response
        .text()
        .await
        .map_err(|e| request_error(step, e))?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<ErrorResponse>(&raw) {
            if let Some(message) = error.error_message {
                return Err(MinecraftAuthError::ElyError { step, message });
            }
        }
        return Err(MinecraftAuthError::ServiceError {
            step,
            status_code: status,
        });
    }

    serde_json::from_str(&raw).map_err(|source| MinecraftAuthError::DeserializeError {
        step,
        raw,
        source,
        status_code: status,
    })
}

fn account_from_response(response: AuthResponse) -> Result<MinecraftAccount, MinecraftAuthError> {
    let id = Uuid::parse_str(&response.selected_profile.id)
        .map_err(|_| MinecraftAuthError::ElyError {
            step: MinecraftAuthStep::ElyAuthenticate,
            message: "Ely.by returned an invalid Minecraft UUID.".to_string(),
        })?;

    Ok(MinecraftAccount {
        id,
        username: response.selected_profile.name,
        access_token: response.access_token,
        refresh_token: String::new(),
        client_token: response.client_token,
        expires: Utc::now() + Duration::hours(24),
        kind: AccountKind::ElyBy,
    })
}

pub async fn authenticate(
    client: &Client,
    username: &str,
    password: &str,
) -> Result<MinecraftAccount, MinecraftAuthError> {
    let response: AuthResponse = post(
        client,
        AUTH_URL,
        json!({
            "username": username,
            "password": password,
            "clientToken": Uuid::new_v4().to_string(),
            "requestUser": true
        }),
        MinecraftAuthStep::ElyAuthenticate,
    )
    .await?;

    account_from_response(response)
}

pub async fn refresh(
    client: &Client,
    account: &MinecraftAccount,
) -> Result<MinecraftAccount, MinecraftAuthError> {
    let response: AuthResponse = post(
        client,
        REFRESH_URL,
        json!({
            "accessToken": account.access_token,
            "clientToken": account.client_token,
            "requestUser": true
        }),
        MinecraftAuthStep::ElyRefresh,
    )
    .await?;

    account_from_response(response)
}
