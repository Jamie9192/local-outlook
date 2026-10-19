use crate::{model::*, providers, store::Store};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};
use tauri::{Emitter, State};

pub struct AppState {
    pub store: Store,
    pub client: reqwest::Client,
    pub refresh: tokio::sync::Mutex<()>,
}
impl AppState {
    pub fn new(dir: &Path) -> Result<Self, String> {
        let store = Store::new(dir)?;
        store.prune()?;
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(8))
            .timeout(std::time::Duration::from_secs(25))
            .gzip(true)
            .user_agent("LocalOutlook/0.1")
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .map_err(|_| "Could not initialise networking.")?;
        Ok(Self {
            store,
            client,
            refresh: tokio::sync::Mutex::new(()),
        })
    }
}

fn key_entry(id: &str) -> Result<keyring::Entry, String> {
    if !matches!(id, "ticketmaster" | "cinema" | "skiddle") {
        return Err("This source does not use an API key.".into());
    }
    keyring::Entry::new("uk.localoutlook.desktop", id)
        .map_err(|_| "Could not open the system credential store.".to_string())
}
fn secret(id: &str) -> Result<Option<String>, String> {
    match key_entry(id)?.get_password() {
        Ok(key)=>Ok(Some(key)),Err(keyring::Error::NoEntry)=>Ok(None),
        Err(_)=>Err("The system credential store could not be read. Check Keychain or Windows Credential Manager.".into())
    }
}
fn key_sources(store: &Store) -> Result<Vec<String>, String> {
    Ok(store
        .get::<Vec<String>>("credential-status")?
        .map(|(v, _)| v)
        .unwrap_or_default())
}
fn source_key(
    id: &str,
    location: &Location,
    settings: &Settings,
    _from: &str,
    _through: &str,
) -> String {
    format!(
        "source:v4:{id}:{}:{}:{}:{}",
        location.postcode,
        settings.radius_miles,
        settings.lookahead_days,
        if id == "cinema" {
            settings.cinema_source.as_str()
        } else if id == "weather" {
            settings.weather_contact.as_str()
        } else {
            ""
        }
    )
}
pub fn is_stale(time: &str, now: DateTime<Utc>) -> bool {
    time.parse::<DateTime<Utc>>()
        .map(|t| now - t >= Duration::hours(24) || t > now + Duration::minutes(5))
        .unwrap_or(true)
}
fn setup_note(id: &str, settings: &Settings, keys: &[String]) -> Option<String> {
    if (matches!(id, "ticketmaster" | "skiddle")
        || id == "cinema" && settings.cinema_source == "tmdb")
        && !keys.iter().any(|k| k == id)
    {
        return Some("Add an API credential in Settings to connect this source.".into());
    }
    if id == "weather" {
        let c = settings.weather_contact.trim();
        if !(c.contains('@') && !c.contains(' ')) && providers::https(c).is_none() {
            return Some("Add a contact email or website in Settings. MET Norway requires identifiable requests.".into());
        }
    }
    None
}

async fn area(
    state: &AppState,
    settings: &Settings,
    network: bool,
    force: bool,
    app: Option<&tauri::AppHandle>,
) -> Result<Option<Snapshot>, String> {
    if settings.postcode.is_empty() {
        return Ok(None);
    }
    let postcode = normalize_postcode(&settings.postcode)?;
    if postcode.starts_with("BT") && !settings.ni_licence_confirmed {
        return Err("Northern Ireland postcode data needs a commercial licence. Confirm it in Settings before searching a BT postcode.".into());
    }
    let geo_key = format!("postcode:{postcode}");
    let cached_location = state.store.get::<Location>(&geo_key)?;
    let location = match cached_location {
        Some((location, time)) if !network || Utc::now() - time < Duration::days(30) => location,
        _ if network => {
            let location = providers::geocode(&state.client, &postcode).await?;
            state.store.put(&geo_key, &location)?;
            location
        }
        _ => return Ok(None),
    };
    let from = Utc::now()
        .with_timezone(&chrono_tz::Europe::London)
        .date_naive();
    let through = from + Duration::days(i64::from(settings.lookahead_days) - 1);
    let keys = key_sources(&state.store)?;
    let mut snapshot = Snapshot {
        location: location.clone(),
        radius_miles: settings.radius_miles,
        lookahead_days: settings.lookahead_days,
        from: from.to_string(),
        through: through.to_string(),
        signals: Vec::new(),
        sources: Vec::new(),
    };
    // Free sources finish first; a slow event feed cannot prevent their results being cached.
    for id in [
        "schools",
        "bank_holidays",
        "weather",
        "ticketmaster",
        "skiddle",
        "cinema",
    ] {
        let mut status = SourceStatus {
            id: id.into(),
            state: "disabled".into(),
            note: "Disabled in Settings.".into(),
            updated_at: None,
            count: 0,
        };
        if !settings.enabled.iter().any(|p| p == id) {
            snapshot.sources.push(status);
            continue;
        }
        if let Some(note) = setup_note(id, settings, &keys) {
            status.state = "needs_setup".into();
            status.note = note;
            snapshot.sources.push(status);
            continue;
        }
        let cache_key = source_key(
            id,
            &location,
            settings,
            &from.to_string(),
            &through.to_string(),
        );
        let cached = state.store.get::<CachedSource>(&cache_key)?.map(|(v, _)| v);
        let expired = cached.as_ref().is_none_or(|c| {
            is_stale(&c.fetched_at, Utc::now())
                || c.from != from.to_string()
                || c.through != through.to_string()
        });
        // Provider Expires headers take precedence over a repeated force-refresh click.
        let can_refresh = cached
            .as_ref()
            .and_then(|c| c.refresh_after.as_ref())
            .and_then(|t| t.parse::<DateTime<Utc>>().ok())
            .is_none_or(|t| Utc::now() >= t);
        let mut selected = cached.clone();
        let mut error = None;
        if network && (force || expired) && can_refresh {
            if let Some(app) = app {
                let _ = app.emit("source-progress", id);
            }
            let credential = if matches!(id, "ticketmaster" | "skiddle")
                || id == "cinema" && settings.cinema_source == "tmdb"
            {
                secret(id)
            } else {
                Ok(None)
            };
            let result = match credential {
                Ok(key) => {
                    providers::fetch(
                        id,
                        &state.client,
                        &state.store,
                        &location,
                        settings,
                        (from, through),
                        key.as_deref().unwrap_or(""),
                    )
                    .await
                }
                Err(e) => Err(e),
            };
            match result {
                Ok(data) => {
                    let c = CachedSource {
                        from: from.to_string(),
                        through: through.to_string(),
                        signals: data.signals,
                        note: data.note,
                        partial: data.partial,
                        fetched_at: Utc::now().to_rfc3339(),
                        refresh_after: data.refresh_after,
                    };
                    state.store.put(&cache_key, &c)?;
                    selected = Some(c);
                }
                Err(e) => error = Some(e),
            }
        }
        if let Some(mut c) = selected {
            c.signals.retain(|s| {
                s.date <= through.to_string()
                    && s.end_date.as_deref().unwrap_or(&s.date) >= from.to_string().as_str()
            });
            status.state = if error.is_some() {
                "error"
            } else if is_stale(&c.fetched_at, Utc::now())
                || c.from != from.to_string()
                || c.through != through.to_string()
            {
                "stale"
            } else if c.partial {
                "partial"
            } else {
                "fresh"
            }
            .into();
            status.count = c.signals.len();
            status.updated_at = Some(c.fetched_at);
            status.note = error
                .map(|e| format!("{e} Showing the last successful cached data. {}", c.note))
                .unwrap_or(c.note);
            snapshot.signals.extend(c.signals);
        } else {
            status.state = if error.is_some() {
                "error"
            } else {
                "not_loaded"
            }
            .into();
            status.note = error.unwrap_or_else(|| {
                "Not checked for this area and date range. Press Refresh to load this source."
                    .into()
            });
        }
        snapshot.sources.push(status);
    }
    snapshot
        .signals
        .sort_by(|a, b| (&a.date, &a.time, &a.title).cmp(&(&b.date, &b.time, &b.title)));
    let mut seen = HashSet::new();
    snapshot.signals.retain(|s| {
        if s.category != "events" {
            return true;
        }
        seen.insert((
            s.title.to_lowercase(),
            s.date.clone(),
            s.time.clone(),
            s.venue.as_ref().map(|v| v.to_lowercase()),
        ))
    });
    Ok(Some(snapshot))
}

#[derive(Serialize)]
pub struct Bootstrap {
    settings: Settings,
    credentials: Vec<String>,
    snapshot: Option<Snapshot>,
}
#[tauri::command]
pub async fn bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    let settings = state.store.settings()?;
    let credentials = key_sources(&state.store)?;
    let snapshot = area(&state, &settings, false, false, None).await?;
    Ok(Bootstrap {
        settings,
        credentials,
        snapshot,
    })
}

#[tauri::command]
pub async fn check_area(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    postcode: String,
    force: bool,
) -> Result<Snapshot, String> {
    let _guard = state
        .refresh
        .try_lock()
        .map_err(|_| "A search is already running. Wait for it to finish.")?;
    let mut settings = state.store.settings()?;
    settings.postcode = normalize_postcode(&postcode)?;
    let result = area(&state, &settings, true, force, Some(&app))
        .await?
        .ok_or("Enter a postcode first.")?;
    state.store.save_settings(&settings)?;
    Ok(result)
}

#[derive(Deserialize)]
pub struct SaveRequest {
    settings: Settings,
    secrets: BTreeMap<String, String>,
    remove_secrets: Vec<String>,
}
#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    request: SaveRequest,
) -> Result<Bootstrap, String> {
    let _guard = state
        .refresh
        .try_lock()
        .map_err(|_| "Wait for the current search to finish before saving settings.")?;
    let mut settings = request.settings;
    settings.validate()?;
    if !settings.postcode.trim().is_empty() {
        settings.postcode = normalize_postcode(&settings.postcode)?;
    }
    for (id, key) in &request.secrets {
        key_entry(id)?;
        if key.trim().is_empty() || key.len() > 4096 || key.chars().any(char::is_control) {
            return Err("Enter a valid API credential without line breaks.".into());
        }
    }
    for id in &request.remove_secrets {
        key_entry(id)?;
    }
    let mut credentials = key_sources(&state.store)?;
    for (id, key) in request.secrets {
        key_entry(&id)?.set_password(key.trim()).map_err(|_|"Could not save the key in the system credential store. Nothing was written to a plain-text file.")?;
        if !credentials.contains(&id) {
            credentials.push(id);
        }
    }
    for id in request.remove_secrets {
        match key_entry(&id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => {
                return Err("Could not remove the key from the system credential store.".into())
            }
        }
        credentials.retain(|c| c != &id);
    }
    state.store.put("credential-status", &credentials)?;
    state.store.save_settings(&settings)?;
    let snapshot = area(&state, &settings, false, false, None).await?;
    Ok(Bootstrap {
        settings,
        credentials,
        snapshot,
    })
}

#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> Result<(), String> {
    let _guard = state
        .refresh
        .try_lock()
        .map_err(|_| "Wait for the current search to finish before clearing the cache.")?;
    let credentials = key_sources(&state.store)?;
    state.store.clear_cache()?;
    state.store.put("credential-status", &credentials)?;
    Ok(())
}

#[tauri::command]
pub async fn test_provider(
    state: State<'_, AppState>,
    id: String,
    key: Option<String>,
    contact: Option<String>,
    cinema_source: Option<String>,
) -> Result<String, String> {
    let _guard = state
        .refresh
        .try_lock()
        .map_err(|_| "Wait for the current request to finish.")?;
    if id == "cinema" && cinema_source.as_deref().unwrap_or("wikidata") == "wikidata" {
        providers::open_cinema(
            &state.client,
            &Utc::now().date_naive().to_string(),
            &(Utc::now().date_naive() + Duration::days(30)).to_string(),
        )
        .await?;
        return Ok(
            "Wikidata connected. No key needed; community release-date coverage is incomplete."
                .into(),
        );
    }
    let draft = key.filter(|s| !s.trim().is_empty());
    let key = if draft.is_some() {
        draft
    } else if matches!(id.as_str(), "ticketmaster" | "skiddle" | "cinema") {
        secret(&id)?
    } else {
        None
    };
    let key = key.unwrap_or_default();
    if matches!(id.as_str(), "ticketmaster" | "skiddle" | "cinema") && key.is_empty() {
        return Err("Enter a credential first, then test the connection.".into());
    }
    let request = match id.as_str() {
        "ticketmaster" => state
            .client
            .get("https://app.ticketmaster.com/discovery/v2/events.json")
            .query(&[
                ("apikey", key.as_str()),
                ("size", "1"),
                ("countryCode", "GB"),
            ]),
        "skiddle" => state
            .client
            .get("https://www.skiddle.com/api/v1/events/search/")
            .query(&[("api_key", key.as_str()), ("limit", "1")]),
        "cinema" => state
            .client
            .get("https://api.themoviedb.org/3/configuration")
            .bearer_auth(&key),
        "weather" => {
            let contact = contact.unwrap_or_default();
            let settings = Settings {
                weather_contact: contact.clone(),
                ..Settings::default()
            };
            settings.validate()?;
            if let Some(note) = setup_note("weather", &settings, &[]) {
                return Err(note);
            }
            state
                .client
                .get("https://api.met.no/weatherapi/locationforecast/2.0/compact")
                .header("User-Agent", format!("LocalOutlook/0.1 ({contact})"))
                .query(&[("lat", "51.5000"), ("lon", "-0.1000")])
        }
        "schools" => state
            .client
            .get("https://api.housemait.com/api/la-term-dates/stats"),
        "bank_holidays" => state.client.get("https://www.gov.uk/bank-holidays.json"),
        _ => return Err("Unknown source.".into()),
    };
    let r = request
        .send()
        .await
        .map_err(|_| "Could not connect. Check your connection and try again.")?;
    if !r.status().is_success() {
        return Err(format!(
            "Connection returned HTTP {}. Check your credential and provider requirements.",
            r.status().as_u16()
        ));
    }
    let json: serde_json::Value = r
        .json()
        .await
        .map_err(|_| "The source did not return readable data.")?;
    if json["error"].as_i64().unwrap_or(0) != 0
        || json.get("fault").is_some()
        || json["success"].as_bool() == Some(false)
    {
        return Err(
            "The source rejected this credential. Check your account and permissions.".into(),
        );
    }
    Ok("Connection successful. This test does not establish commercial permission.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Makes live requests to Postcodes.io, Housemait and GOV.UK"]
    fn live_public_sources_and_offline_cache() {
        tauri::async_runtime::block_on(async {
            let dir = std::env::temp_dir().join(format!(
                "local-outlook-live-{}",
                Utc::now().timestamp_nanos_opt().unwrap()
            ));
            let mut state = AppState::new(&dir).unwrap();
            let settings = Settings {
                postcode: "SW1A 1AA".into(),
                enabled: vec!["schools".into(), "bank_holidays".into(), "cinema".into()],
                ..Settings::default()
            };
            let live = area(&state, &settings, true, false, None)
                .await
                .unwrap()
                .unwrap();
            for source in live
                .sources
                .iter()
                .filter(|s| settings.enabled.contains(&s.id))
            {
                assert!(
                    matches!(source.state.as_str(), "fresh" | "partial"),
                    "{}: {}",
                    source.id,
                    source.note
                );
            }
            assert!(
                live.signals.iter().any(|s| s.category == "schools"),
                "Expected a council holiday in the next 30 days"
            );
            state.client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all("http://127.0.0.1:1").unwrap())
                .connect_timeout(std::time::Duration::from_millis(300))
                .build()
                .unwrap();
            let cached = area(&state, &settings, false, false, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(cached.signals.len(), live.signals.len());
            let offline = area(&state, &settings, true, true, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(offline.signals.len(), live.signals.len());
            for source in offline
                .sources
                .iter()
                .filter(|s| settings.enabled.contains(&s.id))
            {
                assert_eq!(source.state, "error");
                assert_eq!(
                    source.updated_at,
                    live.sources
                        .iter()
                        .find(|s| s.id == source.id)
                        .unwrap()
                        .updated_at
                );
            }
            drop(state);
            std::fs::remove_dir_all(dir).unwrap();
        });
    }
    #[test]
    fn stale_threshold_is_exact_and_corrupt_timestamps_are_stale() {
        let now = Utc::now();
        assert!(!is_stale(&(now - Duration::hours(23)).to_rfc3339(), now));
        assert!(is_stale(&(now - Duration::hours(24)).to_rfc3339(), now));
        assert!(is_stale("broken", now));
    }
    #[test]
    fn cached_queries_are_separate_for_area_radius_and_dates() {
        let l = Location {
            postcode: "SW1A 1AA".into(),
            latitude: 51.5,
            longitude: -0.1,
            council: "Westminster".into(),
            country: "England".into(),
        };
        let a = Settings::default();
        let mut b = a.clone();
        b.radius_miles = 6.0;
        assert_ne!(
            source_key("ticketmaster", &l, &a, "2026-10-19", "2026-11-17"),
            source_key("ticketmaster", &l, &b, "2026-10-19", "2026-11-17")
        );
    }
}
