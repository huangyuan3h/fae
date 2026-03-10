use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::info;
use crate::models::system_settings::{
    SystemSettings, LocationInfo, UpdateSystemSettingsRequest,
    IpLocationResponse,
};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

const SETTINGS_KEY: &str = "system.settings";

pub async fn get_system_settings_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SystemSettings>>, StatusCode> {
    match get_system_settings(&state.db_pool).await {
        Ok(settings) => Ok(Json(ApiResponse {
            ok: true,
            data: Some(settings),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get system settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_system_settings_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSystemSettingsRequest>,
) -> Result<Json<ApiResponse<SystemSettings>>, StatusCode> {
    let location = payload.location.map(|loc| LocationInfo {
        city: loc.city,
        country: loc.country,
        country_code: loc.country_code,
        latitude: loc.latitude,
        longitude: loc.longitude,
    });
    
    let settings = SystemSettings::new(payload.timezone, location);
    
    match save_system_settings(&state.db_pool, &settings).await {
        Ok(saved) => {
            info!("System settings updated successfully");
            Ok(Json(ApiResponse {
                ok: true,
                data: Some(saved),
                error: None,
            }))
        },
        Err(e) => {
            tracing::error!("Failed to save system settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn detect_location_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<IpLocationResponse>>, StatusCode> {
    match detect_location_by_ip().await {
        Ok(location) => {
            if let Some(ref loc) = location {
                let settings = SystemSettings::new(
                    loc.timezone.clone(),
                    Some(LocationInfo {
                        city: loc.city.clone(),
                        country: loc.country.clone(),
                        country_code: loc.country_code.clone(),
                        latitude: Some(loc.latitude),
                        longitude: Some(loc.longitude),
                    }),
                );
                let _ = save_system_settings(&state.db_pool, &settings).await;
            }
            
            Ok(Json(ApiResponse {
                ok: true,
                data: location,
                error: None,
            }))
        },
        Err(e) => {
            tracing::error!("Failed to detect location: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_system_settings(
    db_pool: &sqlx::SqlitePool,
) -> Result<SystemSettings, sqlx::Error> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(SETTINGS_KEY)
        .fetch_optional(db_pool)
        .await?;

    match row {
        Some(row) => {
            let value: String = row.try_get("value")?;
            match serde_json::from_str::<SystemSettings>(&value) {
                Ok(settings) => Ok(settings),
                Err(_) => Ok(SystemSettings::default()),
            }
        },
        None => Ok(SystemSettings::default()),
    }
}

pub async fn save_system_settings(
    db_pool: &sqlx::SqlitePool,
    settings: &SystemSettings,
) -> Result<SystemSettings, sqlx::Error> {
    let value = serde_json::to_string(settings)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to serialize settings: {}", e)))?;
    
    sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, strftime('%s', 'now'))")
        .bind(SETTINGS_KEY)
        .bind(&value)
        .execute(db_pool)
        .await?;

    Ok(settings.clone())
}

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    city: Option<String>,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
}

async fn detect_location_by_ip() -> Result<Option<IpLocationResponse>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("http://ip-api.com/json/")
        .send()
        .await
        .map_err(|e| format!("Failed to call IP API: {}", e))?;

    let api_response: IpApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse IP API response: {}", e))?;

    if api_response.status != "success" {
        return Ok(None);
    }

    Ok(Some(IpLocationResponse {
        city: api_response.city.unwrap_or_default(),
        country: api_response.country.unwrap_or_default(),
        country_code: api_response.country_code.unwrap_or_default(),
        latitude: api_response.lat.unwrap_or(0.0),
        longitude: api_response.lon.unwrap_or(0.0),
        timezone: api_response.timezone.unwrap_or_else(|| "UTC".to_string()),
    }))
}