use chrono::{DateTime, Utc};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct SystemContext {
    pub current_time: DateTime<Utc>,
    pub timezone: String,
    pub location: Option<LocationContext>,
}

#[derive(Debug, Clone)]
pub struct LocationContext {
    pub city: String,
    pub country: String,
    pub country_code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for SystemContext {
    fn default() -> Self {
        Self {
            current_time: Utc::now(),
            timezone: "UTC".to_string(),
            location: None,
        }
    }
}

impl SystemContext {
    pub fn new(timezone: String, location: Option<LocationContext>) -> Self {
        Self {
            current_time: Utc::now(),
            timezone,
            location,
        }
    }

    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = timezone;
        self
    }

    pub fn with_location(mut self, location: LocationContext) -> Self {
        self.location = Some(location);
        self
    }

    pub fn to_prompt_section(&self) -> String {
        let mut parts = Vec::new();

        let local_time = self.get_local_time();
        let formatted_time = local_time.format("%Y-%m-%d %H:%M:%S %:z").to_string();
        parts.push(format!("Current date and time: {}", formatted_time));

        if let Some(ref loc) = self.location {
            let location_str = if loc.latitude.is_some() && loc.longitude.is_some() {
                format!(
                    "{}, {} ({}, {})",
                    loc.city,
                    loc.country,
                    loc.latitude.unwrap(),
                    loc.longitude.unwrap()
                )
            } else {
                format!("{}, {}", loc.city, loc.country)
            };
            parts.push(format!("User location: {}", location_str));
        }

        parts.push(format!("Timezone: {}", self.timezone));

        format!("## System Context\n\n{}\n", parts.join("\n"))
    }

    fn get_local_time(&self) -> DateTime<Tz> {
        if let Ok(tz) = self.timezone.parse::<Tz>() {
            self.current_time.with_timezone(&tz)
        } else {
            self.current_time.with_timezone(&chrono_tz::UTC)
        }
    }

    pub fn refresh_time(&mut self) {
        self.current_time = Utc::now();
    }
}

impl From<crate::models::system_settings::SystemSettings> for SystemContext {
    fn from(settings: crate::models::system_settings::SystemSettings) -> Self {
        let location = settings.location.map(|loc| LocationContext {
            city: loc.city,
            country: loc.country,
            country_code: loc.country_code,
            latitude: loc.latitude,
            longitude: loc.longitude,
        });

        SystemContext::new(settings.timezone, location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_context_default() {
        let ctx = SystemContext::default();
        assert_eq!(ctx.timezone, "UTC");
        assert!(ctx.location.is_none());
    }

    #[test]
    fn test_system_context_with_location() {
        let ctx = SystemContext::default()
            .with_timezone("Asia/Shanghai".to_string())
            .with_location(LocationContext {
                city: "Shanghai".to_string(),
                country: "China".to_string(),
                country_code: "CN".to_string(),
                latitude: Some(31.2304),
                longitude: Some(121.4737),
            });

        assert_eq!(ctx.timezone, "Asia/Shanghai");
        assert!(ctx.location.is_some());
    }

    #[test]
    fn test_to_prompt_section() {
        let ctx = SystemContext::default()
            .with_timezone("Asia/Shanghai".to_string())
            .with_location(LocationContext {
                city: "Shanghai".to_string(),
                country: "China".to_string(),
                country_code: "CN".to_string(),
                latitude: Some(31.2304),
                longitude: Some(121.4737),
            });

        let prompt = ctx.to_prompt_section();
        assert!(prompt.contains("Current date and time"));
        assert!(prompt.contains("User location: Shanghai, China"));
        assert!(prompt.contains("Timezone: Asia/Shanghai"));
    }

    #[test]
    fn test_to_prompt_section_without_location() {
        let ctx = SystemContext::default().with_timezone("UTC".to_string());
        let prompt = ctx.to_prompt_section();

        assert!(prompt.contains("Current date and time"));
        assert!(!prompt.contains("User location"));
        assert!(prompt.contains("Timezone: UTC"));
    }
}
