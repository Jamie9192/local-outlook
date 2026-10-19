use serde::{Deserialize, Serialize};

pub const PROVIDERS: [&str; 6] = [
    "ticketmaster",
    "skiddle",
    "cinema",
    "schools",
    "weather",
    "bank_holidays",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub postcode: String,
    pub radius_miles: f64,
    pub lookahead_days: u32,
    pub enabled: Vec<String>,
    pub weather_contact: String,
    pub cinema_source: String,
    pub ni_licence_confirmed: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            postcode: String::new(),
            radius_miles: 5.0,
            lookahead_days: 30,
            enabled: vec![
                "ticketmaster",
                "cinema",
                "schools",
                "weather",
                "bank_holidays",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            weather_contact: String::new(),
            cinema_source: "wikidata".into(),
            ni_licence_confirmed: false,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.cinema_source.as_str(), "wikidata" | "tmdb") {
            return Err("Choose a valid cinema source.".into());
        }
        if !self.radius_miles.is_finite() || !(0.5..=100.0).contains(&self.radius_miles) {
            return Err("Choose a radius between 0.5 and 100 miles.".into());
        }
        if !(1..=90).contains(&self.lookahead_days) {
            return Err("Choose between 1 and 90 days ahead.".into());
        }
        if self
            .enabled
            .iter()
            .any(|p| !PROVIDERS.contains(&p.as_str()))
        {
            return Err("Unknown data source.".into());
        }
        if self.weather_contact.len() > 250 || self.weather_contact.chars().any(char::is_control) {
            return Err("Enter a valid weather contact.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub postcode: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub council: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub category: String,
    pub source: String,
    pub title: String,
    pub date: String,
    pub end_date: Option<String>,
    pub time: Option<String>,
    pub venue: Option<String>,
    pub distance_miles: Option<f64>,
    pub url: Option<String>,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceStatus {
    pub id: String,
    pub state: String,
    pub note: String,
    pub updated_at: Option<String>,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub location: Location,
    pub radius_miles: f64,
    pub lookahead_days: u32,
    pub from: String,
    pub through: String,
    pub signals: Vec<Signal>,
    pub sources: Vec<SourceStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedSource {
    pub from: String,
    pub through: String,
    pub signals: Vec<Signal>,
    pub note: String,
    pub partial: bool,
    pub fetched_at: String,
    pub refresh_after: Option<String>,
}

pub struct ProviderData {
    pub signals: Vec<Signal>,
    pub note: String,
    pub partial: bool,
    pub refresh_after: Option<String>,
}
impl ProviderData {
    pub fn new(signals: Vec<Signal>, note: impl Into<String>) -> Self {
        Self {
            signals,
            note: note.into(),
            partial: false,
            refresh_after: None,
        }
    }
}

pub fn normalize_postcode(input: &str) -> Result<String, String> {
    let code: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();
    if !(5..=7).contains(&code.len()) || !code.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Enter a full UK postcode, for example SW1A 1AA.".into());
    }
    let at = code.len() - 3;
    let end = &code.as_bytes()[at..];
    if !code.as_bytes()[0].is_ascii_alphabetic()
        || !end[0].is_ascii_digit()
        || !end[1..].iter().all(u8::is_ascii_alphabetic)
    {
        return Err("Enter a full UK postcode, for example SW1A 1AA.".into());
    }
    Ok(format!("{} {}", &code[..at], &code[at..]))
}

pub fn distance_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let a = ((lat2 - lat1).to_radians() / 2.0).sin().powi(2)
        + lat1.to_radians().cos()
            * lat2.to_radians().cos()
            * ((lon2 - lon1).to_radians() / 2.0).sin().powi(2);
    3958.7613 * 2.0 * a.clamp(0.0, 1.0).sqrt().asin()
}

pub fn geohash(lat: f64, lon: f64) -> String {
    let alphabet = b"0123456789bcdefghjkmnpqrstuvwxyz";
    let mut ranges = [[-180.0, 180.0], [-90.0, 90.0]];
    let mut hash = String::new();
    let mut value = 0;
    for bit in 0..45 {
        let axis = bit % 2;
        let coordinate = if axis == 0 { lon } else { lat };
        let mid = (ranges[axis][0] + ranges[axis][1]) / 2.0;
        value <<= 1;
        if coordinate >= mid {
            value |= 1;
            ranges[axis][0] = mid;
        } else {
            ranges[axis][1] = mid;
        }
        if bit % 5 == 4 {
            hash.push(alphabet[value] as char);
            value = 0;
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn postcodes_are_normalised_and_untrusted_input_is_rejected() {
        assert_eq!(normalize_postcode(" sw1a1aa ").unwrap(), "SW1A 1AA");
        for code in ["", "SW1", "BT1<script>", "123 456", "éé1 1AA"] {
            assert!(normalize_postcode(code).is_err());
        }
    }
    #[test]
    fn distance_and_geohash_are_known_values() {
        assert!(distance_miles(51.5, -0.1, 51.5, -0.1) < 0.00001);
        assert!((distance_miles(51.5, -0.1, 51.5723658, -0.1) - 5.0).abs() < 0.002);
        assert!(geohash(42.6, -5.6).starts_with("ezs42"));
    }
    #[test]
    fn settings_reject_bad_radius() {
        let mut s = Settings {
            radius_miles: f64::NAN,
            ..Settings::default()
        };
        assert!(s.validate().is_err());
        s.radius_miles = 101.0;
        assert!(s.validate().is_err());
    }
}
