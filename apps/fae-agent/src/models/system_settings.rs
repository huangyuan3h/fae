use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    #[serde(rename = "timezone")]
    pub timezone: String,
    #[serde(rename = "location")]
    pub location: Option<LocationInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub city: String,
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for SystemSettings {
    fn default() -> Self {
        SystemSettings {
            timezone: "UTC".to_string(),
            location: None,
        }
    }
}

impl SystemSettings {
    pub fn new(timezone: String, location: Option<LocationInfo>) -> Self {
        SystemSettings { timezone, location }
    }
}

#[derive(Deserialize)]
pub struct UpdateSystemSettingsRequest {
    pub timezone: String,
    pub location: Option<LocationInfoRequest>,
}

#[derive(Deserialize)]
pub struct LocationInfoRequest {
    pub city: String,
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize)]
pub struct IpLocationResponse {
    pub city: String,
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}
