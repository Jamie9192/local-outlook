use crate::{model::*, store::Store};
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

pub fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn number(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str()?.parse().ok())
}
pub fn https(value: &str) -> Option<String> {
    let url = reqwest::Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none())
    .then(|| value.to_string())
}
async fn checked(response: Result<Response, reqwest::Error>) -> Result<Response, String> {
    let r = response.map_err(|e| {
        if e.is_timeout() {
            "The source timed out. Try Refresh again."
        } else {
            "Could not reach the source. Check your connection and try Refresh."
        }
    })?;
    match r.status().as_u16() {
        200..=299 => Ok(r),
        401 | 403 => {
            Err("Access denied. Check the API key and provider requirements in Settings.".into())
        }
        429 => Err("The source's rate limit was reached. Wait before refreshing again.".into()),
        404 => Err("The requested data is not available from this source.".into()),
        _ => Err(format!(
            "The source returned HTTP {}. Try again later.",
            r.status().as_u16()
        )),
    }
}
async fn json(response: Result<Response, reqwest::Error>) -> Result<Value, String> {
    checked(response)
        .await?
        .json()
        .await
        .map_err(|_| "The source returned unreadable data. Try again later.".into())
}
fn signal(id: String, category: &str, source: &str, title: String, date: String) -> Signal {
    Signal {
        id,
        category: category.into(),
        source: source.into(),
        title,
        date,
        end_date: None,
        time: None,
        venue: None,
        distance_miles: None,
        url: None,
        detail: String::new(),
    }
}
fn within_dates(day: &str, from: &str, through: &str) -> bool {
    day >= from && day <= through
}

pub async fn geocode(client: &Client, postcode: &str) -> Result<Location, String> {
    let url = format!(
        "https://api.postcodes.io/postcodes/{}",
        postcode.replace(' ', "")
    );
    let data = json(client.get(url).send().await).await?;
    let p = &data["result"];
    let lat = number(&p["latitude"]).ok_or("This postcode has no verified coordinates.")?;
    let lon = number(&p["longitude"]).ok_or("This postcode has no verified coordinates.")?;
    let county = text(p, "admin_county");
    let council = if text(p, "country") == "Northern Ireland" {
        "Northern Ireland"
    } else if county.is_empty() {
        text(p, "admin_district")
    } else {
        county
    };
    Ok(Location {
        postcode: text(p, "postcode").into(),
        latitude: lat,
        longitude: lon,
        country: text(p, "country").into(),
        council: council.into(),
    })
}

fn ticketmaster_events(
    data: &Value,
    location: &Location,
    radius: f64,
    from: &str,
    through: &str,
) -> (Vec<Signal>, usize) {
    let mut out = Vec::new();
    let mut unknown = 0;
    for event in data["_embedded"]["events"].as_array().into_iter().flatten() {
        let start = &event["dates"]["start"];
        let day = text(start, "localDate");
        if !within_dates(day, from, through) {
            continue;
        }
        let venues = event["_embedded"]["venues"].as_array();
        let nearest = venues
            .into_iter()
            .flatten()
            .filter_map(|v| {
                let lat = number(&v["location"]["latitude"])?;
                let lon = number(&v["location"]["longitude"])?;
                if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
                    return None;
                }
                Some((
                    distance_miles(location.latitude, location.longitude, lat, lon),
                    v,
                ))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((distance, venue)) = nearest else {
            unknown += 1;
            continue;
        };
        if distance > radius {
            continue;
        }
        let mut s = signal(
            format!("tm:{}", text(event, "id")),
            "events",
            "ticketmaster",
            text(event, "name").into(),
            day.into(),
        );
        s.time = (!start["timeTBA"].as_bool().unwrap_or(false)
            && !text(start, "localTime").is_empty())
        .then(|| text(start, "localTime")[..5.min(text(start, "localTime").len())].into());
        s.venue = Some(text(venue, "name").into());
        s.distance_miles = Some(distance);
        s.url = https(text(event, "url"));
        let status = text(&event["dates"]["status"], "code");
        s.detail = match status {
            "cancelled" => "Cancelled — check the source for updates.",
            "postponed" | "rescheduled" => "Date changed — check the source before planning.",
            _ => "Local event. Attendance and effect on sales are not known.",
        }
        .into();
        out.push(s);
    }
    (out, unknown)
}

async fn ticketmaster(
    client: &Client,
    location: &Location,
    radius: f64,
    from: NaiveDate,
    through: NaiveDate,
    key: &str,
) -> Result<ProviderData, String> {
    let mut windows = vec![(from, through)];
    let mut signals = Vec::new();
    let mut unknown = 0;
    let mut calls = 0;
    let mut truncated = false;
    while let Some((start, end)) = windows.pop() {
        let params = vec![
            ("apikey", key.to_string()),
            ("geoPoint", geohash(location.latitude, location.longitude)),
            ("radius", (radius + 0.1).to_string()),
            ("unit", "miles".into()),
            ("countryCode", "GB".into()),
            ("size", "200".into()),
            ("sort", "date,asc".into()),
            (
                "startDateTime",
                format!("{}T00:00:00Z", start - Duration::days(1)),
            ),
            ("endDateTime", format!("{}T23:59:59Z", end)),
        ];
        let mut page = 0;
        loop {
            if calls >= 80 {
                truncated = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(550)).await;
            let data = json(
                client
                    .get("https://app.ticketmaster.com/discovery/v2/events.json")
                    .query(&params)
                    .query(&[("page", page)])
                    .send()
                    .await,
            )
            .await?;
            calls += 1;
            let total = data["page"]["totalElements"]
                .as_u64()
                .ok_or("Ticketmaster returned an unexpected search response.")?;
            if page == 0 && total > 1000 && start < end {
                let mid = start + Duration::days((end - start).num_days() / 2);
                windows.push((start, mid));
                windows.push((mid + Duration::days(1), end));
                break;
            }
            if total > 1000 {
                truncated = true;
            }
            let (events, missing) = ticketmaster_events(
                &data,
                location,
                radius,
                &from.to_string(),
                &through.to_string(),
            );
            signals.extend(events);
            unknown += missing;
            page += 1;
            if page >= data["page"]["totalPages"].as_u64().unwrap_or(1).min(5) {
                break;
            }
        }
        if calls >= 80 {
            break;
        }
    }
    let mut seen = HashSet::new();
    signals.retain(|s| seen.insert(s.id.clone()));
    let mut p = ProviderData::new(signals,format!("Straight-line distance from postcode centre. {unknown} results without verified venue coordinates were excluded. Ticketmaster does not cover every local event."));
    p.partial = truncated || unknown > 0;
    if truncated {
        p.note
            .push_str(" Search limit reached; narrow the date range for more complete results.");
    }
    Ok(p)
}

async fn skiddle(
    client: &Client,
    location: &Location,
    radius: f64,
    from: &str,
    through: &str,
    key: &str,
) -> Result<ProviderData, String> {
    let mut signals = Vec::new();
    let mut unknown = 0;
    let mut truncated = false;
    for page in 0..30 {
        if page > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(550)).await;
        }
        let data = json(
            client
                .get("https://www.skiddle.com/api/v1/events/search/")
                .query(&[
                    ("api_key", key.to_string()),
                    ("latitude", location.latitude.to_string()),
                    ("longitude", location.longitude.to_string()),
                    ("radius", radius.ceil().to_string()),
                    ("minDate", from.into()),
                    ("maxDate", through.into()),
                    ("limit", "100".into()),
                    ("offset", (page * 100).to_string()),
                    ("order", "distance".into()),
                    ("description", "1".into()),
                ])
                .send()
                .await,
        )
        .await?;
        if data["error"].as_u64().unwrap_or(0) != 0 {
            return Err(
                "Skiddle rejected the request. Check your key and commercial approval.".into(),
            );
        }
        let rows = data["results"]
            .as_array()
            .ok_or("Skiddle returned an unexpected response.")?;
        for e in rows {
            let day = text(e, "date");
            if !within_dates(day, from, through) {
                continue;
            }
            let v = &e["venue"];
            let (Some(lat), Some(lon)) = (number(&v["latitude"]), number(&v["longitude"])) else {
                unknown += 1;
                continue;
            };
            if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
                unknown += 1;
                continue;
            }
            let distance = distance_miles(location.latitude, location.longitude, lat, lon);
            if distance > radius {
                continue;
            }
            let mut s = signal(
                format!("sk:{}", e["id"]),
                "events",
                "skiddle",
                text(e, "eventname").into(),
                day.into(),
            );
            s.venue = Some(text(v, "name").into());
            s.distance_miles = Some(distance);
            s.url = https(text(e, "link"));
            s.time = (!text(&e["openingtimes"], "doorsopen").is_empty())
                .then(|| text(&e["openingtimes"], "doorsopen").into());
            s.detail = "Local event. Check the source for status and full details.".into();
            signals.push(s);
        }
        if rows.len() < 100 {
            break;
        }
        if page == 29 {
            truncated = true;
        }
    }
    let mut out=ProviderData::new(signals,format!("Additional local events. {unknown} results without verified coordinates excluded. Commercial approval from Skiddle is required."));
    out.partial = truncated || unknown > 0;
    if truncated {
        out.note
            .push_str(" Search limit reached; narrow the date range.");
    }
    Ok(out)
}

pub fn uk_release_dates(data: &Value, from: &str, through: &str) -> Vec<String> {
    let mut dates = Vec::new();
    for country in data["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| text(c, "iso_3166_1") == "GB")
    {
        for release in country["release_dates"].as_array().into_iter().flatten() {
            if !matches!(release["type"].as_u64(), Some(2 | 3)) {
                continue;
            }
            if let Some(day) = text(release, "release_date").get(..10) {
                if within_dates(day, from, through) {
                    dates.push(day.into());
                }
            }
        }
    }
    dates.sort();
    dates.dedup();
    dates
}
async fn cinema(
    client: &Client,
    store: &Store,
    from: &str,
    through: &str,
    key: &str,
) -> Result<ProviderData, String> {
    let mut films = Vec::new();
    let mut truncated = false;
    for page in 1..=10 {
        let data = json(
            client
                .get("https://api.themoviedb.org/3/discover/movie")
                .bearer_auth(key)
                .query(&[
                    ("region", "GB".to_string()),
                    ("with_release_type", "2|3".into()),
                    ("release_date.gte", from.into()),
                    ("release_date.lte", through.into()),
                    ("language", "en-GB".into()),
                    ("include_adult", "false".into()),
                    ("page", page.to_string()),
                ])
                .send()
                .await,
        )
        .await?;
        let rows = data["results"]
            .as_array()
            .ok_or("TMDB returned an unexpected release response.")?;
        films.extend(rows.clone());
        if page >= data["total_pages"].as_u64().unwrap_or(1) {
            break;
        }
        if page == 10 {
            truncated = true;
        }
    }
    let mut signals = Vec::new();
    let mut seen = HashSet::new();
    for film in films {
        let Some(id) = film["id"].as_u64() else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let cache_key = format!("tmdb-release:{id}");
        let cached = store.get::<Value>(&cache_key)?;
        let releases = if let Some((data, time)) =
            cached.filter(|(_, t)| Utc::now() - *t < Duration::hours(24))
        {
            let _ = time;
            data
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let d = json(
                client
                    .get(format!(
                        "https://api.themoviedb.org/3/movie/{id}/release_dates"
                    ))
                    .bearer_auth(key)
                    .send()
                    .await,
            )
            .await?;
            store.put(&cache_key, &d)?;
            d
        };
        for day in uk_release_dates(&releases, from, through) {
            let mut s = signal(
                format!("film:{id}:{day}"),
                "cinema",
                "cinema",
                text(&film, "title").into(),
                day,
            );
            s.url = Some(format!("https://www.themoviedb.org/movie/{id}"));
            s.detail="UK theatrical release. A national release date does not confirm screenings at a nearby cinema.".into();
            signals.push(s);
        }
    }
    let mut out=ProviderData::new(signals,"UK theatrical dates verified against TMDB's country-specific release records. Commercial licence required.");
    out.partial = truncated;
    if truncated {
        out.note
            .push_str(" Release result limit reached; narrow your date range.");
    }
    Ok(out)
}

fn normal_name(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .replace("cityof", "")
        .replace("londonboroughof", "")
}
async fn school_data(client: &Client, store: &Store, location: &Location) -> Result<Value, String> {
    let key = "school-authorities:v1";
    let index = if let Some((data, _)) = store
        .get::<Vec<Value>>(key)?
        .filter(|(_, t)| Utc::now() - *t < Duration::days(30))
    {
        data
    } else {
        let mut rows = Vec::new();
        for page in 1..=10 {
            let d = json(
                client
                    .get("https://api.housemait.com/api/la-term-dates/authorities")
                    .query(&[("pageSize", 100), ("page", page)])
                    .send()
                    .await,
            )
            .await?;
            let entries = d["rows"]
                .as_array()
                .ok_or("The school calendar directory was unreadable.")?;
            rows.extend(entries.clone());
            if rows.len() as u64 >= d["total"].as_u64().unwrap_or(rows.len() as u64)
                || entries.is_empty()
            {
                break;
            }
        }
        store.put(key, &rows)?;
        rows
    };
    let authority=index.iter().find(|a|normal_name(text(a,"name"))==normal_name(&location.council)).ok_or("No matching education authority was found. Check the council's school calendar directly.")?;
    let slug = text(authority, "slug");
    if slug.is_empty() || !slug.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
        return Err("The school authority identifier was invalid.".into());
    }
    json(
        client
            .get(format!(
                "https://api.housemait.com/api/la-term-dates/authorities/{slug}"
            ))
            .send()
            .await,
    )
    .await
}

pub fn school_signals(
    data: &Value,
    location: &Location,
    from: NaiveDate,
    through: NaiveDate,
) -> Result<ProviderData, String> {
    let years = data["academicYears"]
        .as_array()
        .ok_or("No school calendar was provided for this council.")?;
    let mut rows: Vec<&Value> = years
        .iter()
        .flat_map(|y| y["dates"].as_array().into_iter().flatten())
        .collect();
    rows.sort_by_key(|r| text(r, "date"));
    let starts: Vec<NaiveDate> = rows
        .iter()
        .filter(|r| text(r, "event_type") == "term_start")
        .filter_map(|r| text(r, "date").parse().ok())
        .collect();
    let mut spans: Vec<(NaiveDate, NaiveDate, String)> = Vec::new();
    for row in &rows {
        let Ok(day) = text(row, "date").parse::<NaiveDate>() else {
            continue;
        };
        let kind = text(row, "event_type");
        let label = text(row, "label");
        if kind == "term_end" {
            if let Some(next) = starts.iter().find(|s| **s > day) {
                if (*next - day).num_days() >= 5 && (*next - day).num_days() < 100 {
                    spans.push((
                        day + Duration::days(1),
                        *next - Duration::days(1),
                        "School holiday".into(),
                    ));
                }
            }
        } else if kind.contains("half_term")
            || kind == "school_holiday"
            || label.to_lowercase().contains("school holiday")
        {
            if let Ok(end) = text(row, "end_date").parse::<NaiveDate>() {
                if end >= day {
                    spans.push((day, end, label.into()));
                }
            }
        }
    }
    spans.sort_by_key(|s| s.0);
    let mut merged: Vec<(NaiveDate, NaiveDate, String)> = Vec::new();
    for (start, end, label) in spans {
        if let Some(last) = merged.last_mut() {
            if start <= last.1 + Duration::days(1) {
                last.1 = last.1.max(end);
                if label.to_lowercase().contains("half") {
                    last.2 = "Half term".into();
                }
                continue;
            }
        }
        merged.push((start, end, label));
    }
    let authority = &data["authority"];
    let url = https(text(authority, "source_url"));
    let status = text(authority, "import_status");
    if status == "failed" || rows.is_empty() {
        return Err(
            "School dates are unavailable for this council. Check its published calendar.".into(),
        );
    }
    let calendar_start = rows
        .first()
        .and_then(|r| text(r, "date").parse::<NaiveDate>().ok());
    let calendar_end = rows.last().and_then(|r| {
        text(r, "end_date")
            .parse::<NaiveDate>()
            .ok()
            .or_else(|| text(r, "date").parse().ok())
    });
    let mut out=ProviderData::new(Vec::new(),format!("{} council calendar, via Housemait. Individual schools and INSET days may differ. Source imported: {}.",location.council,text(authority,"last_imported_at").get(..10).unwrap_or("unknown")));
    out.partial = status != "ok"
        || calendar_start.is_none_or(|d| d > from)
        || calendar_end.is_none_or(|d| d < through);
    if out.partial {
        out.note
            .push_str(" Some requested dates may be outside available council coverage.");
    }
    for (start, end, label) in merged {
        if start > through || end < from {
            continue;
        }
        let mut s = signal(
            format!("school:{}:{start}", location.council),
            "schools",
            "schools",
            label,
            start.to_string(),
        );
        s.end_date = Some(end.to_string());
        s.url = url.clone();
        s.detail=format!("{} council holiday dates. Schools can set different dates; weekends may be included in the span.",location.council);
        out.signals.push(s);
    }
    Ok(out)
}

async fn weather(
    client: &Client,
    location: &Location,
    from: &str,
    through: &str,
    contact: &str,
) -> Result<ProviderData, String> {
    let r = checked(
        client
            .get("https://api.met.no/weatherapi/locationforecast/2.0/compact")
            .header("User-Agent", format!("LocalOutlook/0.1 ({contact})"))
            .query(&[
                ("lat", format!("{:.4}", location.latitude)),
                ("lon", format!("{:.4}", location.longitude)),
            ])
            .send()
            .await,
    )
    .await?;
    let expires = r
        .headers()
        .get("Expires")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| DateTime::parse_from_rfc2822(v).ok())
        .map(|v| v.with_timezone(&Utc).to_rfc3339());
    let data: Value = r.json().await.map_err(|_| "Weather data was unreadable.")?;
    let rows = data["properties"]["timeseries"]
        .as_array()
        .ok_or("The weather source returned no forecast.")?;
    let mut days: BTreeMap<String, (f64, f64, f64, String)> = BTreeMap::new();
    for row in rows {
        let Ok(time) = text(row, "time").parse::<DateTime<Utc>>() else {
            continue;
        };
        let time = time.with_timezone(&chrono_tz::Europe::London);
        let day = time.date_naive().to_string();
        if !within_dates(&day, from, through) {
            continue;
        }
        let Some(temp) = number(&row["data"]["instant"]["details"]["air_temperature"]) else {
            continue;
        };
        let next = if row["data"]["next_1_hours"].is_object() {
            &row["data"]["next_1_hours"]
        } else {
            &row["data"]["next_6_hours"]
        };
        let symbol = text(&next["summary"], "symbol_code")
            .split('_')
            .next()
            .unwrap_or("")
            .to_string();
        let rain = number(&next["details"]["precipitation_amount"]).unwrap_or(0.0);
        let entry = days.entry(day).or_insert((temp, temp, 0.0, symbol.clone()));
        entry.0 = entry.0.min(temp);
        entry.1 = entry.1.max(temp);
        entry.2 += rain;
        if (11..=14).contains(&time.hour()) {
            entry.3 = symbol;
        }
    }
    let horizon = days.keys().last().cloned().unwrap_or_default();
    let signals=days.into_iter().map(|(day,(low,high,rain,symbol))| {let mut s=signal(format!("weather:{day}"),"weather","weather",format!("{low:.0}–{high:.0}°C · {}",weather_label(&symbol)),day);s.detail=format!("{rain:.1} mm forecast precipitation across available intervals. Temperature range uses available forecast points. Data from MET Norway.");s.url=Some("https://api.met.no/weatherapi/locationforecast/2.0/documentation".into());s}).collect();
    let mut out=ProviderData::new(signals,format!("MET Norway forecast available through {horizon}. Later dates have no weather forecast."));
    out.refresh_after = expires;
    Ok(out)
}
fn weather_label(code: &str) -> String {
    match code {
        "clearsky" => "Clear sky",
        "fair" => "Mostly clear",
        "partlycloudy" => "Partly cloudy",
        "cloudy" => "Cloudy",
        "lightrain" => "Light rain",
        "rain" => "Rain",
        "heavyrain" => "Heavy rain",
        "lightrainshowers" => "Light showers",
        "rainshowers" => "Showers",
        "fog" => "Fog",
        "snow" => "Snow",
        "sleet" => "Sleet",
        _ => code,
    }
    .into()
}

async fn bank_holidays(
    client: &Client,
    location: &Location,
    from: &str,
    through: &str,
) -> Result<ProviderData, String> {
    let data = json(
        client
            .get("https://www.gov.uk/bank-holidays.json")
            .send()
            .await,
    )
    .await?;
    let division = match location.country.as_str() {
        "Scotland" => "scotland",
        "Northern Ireland" => "northern-ireland",
        _ => "england-and-wales",
    };
    let rows = data[division]["events"]
        .as_array()
        .ok_or("Bank holiday dates were unavailable.")?;
    let signals = rows
        .iter()
        .filter(|r| within_dates(text(r, "date"), from, through))
        .map(|r| {
            let day = text(r, "date");
            let mut s = signal(
                format!("bank:{division}:{day}"),
                "bank_holidays",
                "bank_holidays",
                text(r, "title").into(),
                day.into(),
            );
            s.url = Some("https://www.gov.uk/bank-holidays".into());
            s.detail = format!("{} bank holiday calendar.", location.country);
            s
        })
        .collect();
    Ok(ProviderData::new(signals,"Official GOV.UK bank holiday dates. Contains public sector information licensed under the Open Government Licence v3.0."))
}

pub async fn fetch(
    id: &str,
    client: &Client,
    store: &Store,
    location: &Location,
    settings: &Settings,
    window: (NaiveDate, NaiveDate),
    key: &str,
) -> Result<ProviderData, String> {
    let (from, through) = window;
    match id {
        "ticketmaster" => {
            ticketmaster(client, location, settings.radius_miles, from, through, key).await
        }
        "skiddle" => {
            skiddle(
                client,
                location,
                settings.radius_miles,
                &from.to_string(),
                &through.to_string(),
                key,
            )
            .await
        }
        "cinema" if settings.cinema_source == "wikidata" => {
            open_cinema(client, &from.to_string(), &through.to_string()).await
        }
        "cinema" => cinema(client, store, &from.to_string(), &through.to_string(), key).await,
        "schools" => school_signals(
            &school_data(client, store, location).await?,
            location,
            from,
            through,
        ),
        "weather" => {
            weather(
                client,
                location,
                &from.to_string(),
                &through.to_string(),
                &settings.weather_contact,
            )
            .await
        }
        "bank_holidays" => {
            bank_holidays(client, location, &from.to_string(), &through.to_string()).await
        }
        _ => Err("Unknown source.".into()),
    }
}

pub fn open_cinema_signals(
    data: &Value,
    from: &str,
    through: &str,
) -> Result<ProviderData, String> {
    let rows = data["results"]["bindings"]
        .as_array()
        .ok_or("Wikidata returned an unexpected response.")?;
    let mut signals = Vec::new();
    let mut seen = HashSet::new();
    for row in rows {
        let country = text(&row["country"], "value");
        if !country.ends_with("/Q145") || text(&row["precision"], "value") != "11" {
            continue;
        }
        let Some(day) = text(&row["date"], "value").get(..10) else {
            continue;
        };
        if !within_dates(day, from, through) {
            continue;
        }
        let item = text(&row["film"], "value").rsplit('/').next().unwrap_or("");
        if !item.starts_with('Q') || !item[1..].bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let title = text(&row["filmLabel"], "value");
        if title.is_empty() || title == item {
            continue;
        }
        let format = text(&row["formatLabel"], "value").to_lowercase();
        let theatrical = format.contains("theatrical") || format.contains("cinema");
        // Unknown format is disclosed; an explicitly non-theatrical format is excluded.
        if !format.is_empty() && !theatrical {
            continue;
        }
        let id = format!("wikidata:{item}:{day}");
        if !seen.insert(id.clone()) {
            continue;
        }
        let mut s = signal(id, "cinema", "cinema", title.into(), day.into());
        s.url = Some(format!("https://www.wikidata.org/wiki/{item}"));
        s.detail=if theatrical {"UK theatrical date from Wikidata (CC0). Community data can be incomplete or outdated; check the distributor or local cinema."} else {"UK-tagged film publication date from Wikidata (CC0). The release format is not specified, so this is not a confirmed theatrical date. Community coverage is incomplete; verify with the FDA calendar or cinema."}.into();
        signals.push(s);
    }
    let mut out=ProviderData::new(signals,"Free Wikidata structured data (CC0). UK release-date coverage is incomplete and some release formats are unconfirmed. Check the FDA industry calendar or your cinema before planning. No API key or commercial API subscription is required.");
    out.partial = true;
    Ok(out)
}

pub async fn open_cinema(
    client: &Client,
    from: &str,
    through: &str,
) -> Result<ProviderData, String> {
    let query = format!(
        r#"SELECT DISTINCT ?film ?filmLabel ?date ?precision ?country ?formatLabel WHERE {{
      VALUES ?country {{ wd:Q145 }}
      ?release pq:P291 ?country; ps:P577 ?date; psv:P577 ?timeValue.
      FILTER(?date >= "{from}T00:00:00Z"^^xsd:dateTime && ?date <= "{through}T23:59:59Z"^^xsd:dateTime)
      ?timeValue wikibase:timePrecision ?precision. FILTER(?precision = 11)
      FILTER NOT EXISTS {{ ?release wikibase:rank wikibase:DeprecatedRank }}
      ?film p:P577 ?release; wdt:P31/wdt:P279* wd:Q11424.
      OPTIONAL {{ ?release pq:P437 ?format }}
      SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en". }}
    }} ORDER BY ?date LIMIT 500"#
    );
    let data = json(
        client
            .get("https://query.wikidata.org/sparql")
            .header("Accept", "application/sparql-results+json")
            .header(
                "User-Agent",
                "LocalOutlook/0.1 (local desktop sales-planning application)",
            )
            .query(&[("query", query.as_str()), ("format", "json")])
            .send()
            .await,
    )
    .await?;
    open_cinema_signals(&data, from, through)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn open_dates_require_a_precise_uk_date_and_reject_explicit_streaming() {
        let row = |country: &str, precision: &str, format: &str| json!({"country":{"value":format!("http://www.wikidata.org/entity/{country}")},"precision":{"value":precision},"date":{"value":"2026-10-23T00:00:00Z"},"film":{"value":"http://www.wikidata.org/entity/Q123"},"filmLabel":{"value":"A film"},"formatLabel":{"value":format}});
        let data = json!({"results":{"bindings":[row("Q145","11",""),row("Q30","11",""),row("Q145","10",""),row("Q145","11","video on demand")]}});
        let out = open_cinema_signals(&data, "2026-10-19", "2026-10-30").unwrap();
        assert_eq!(out.signals.len(), 1);
        assert!(out.partial);
        assert!(out.signals[0]
            .detail
            .contains("not a confirmed theatrical date"));
    }
    fn location() -> Location {
        Location {
            postcode: "SW1A 1AA".into(),
            latitude: 51.5,
            longitude: -0.1,
            country: "England".into(),
            council: "Westminster".into(),
        }
    }
    #[test]
    fn uk_releases_exclude_streaming_and_foreign_dates() {
        let d = json!({"results":[{"iso_3166_1":"US","release_dates":[{"type":3,"release_date":"2026-10-20T00:00:00Z"}]},{"iso_3166_1":"GB","release_dates":[{"type":4,"release_date":"2026-10-21T00:00:00Z"},{"type":3,"release_date":"2026-10-23T00:00:00Z"}]}]});
        assert_eq!(
            uk_release_dates(&d, "2026-10-19", "2026-10-30"),
            vec!["2026-10-23"]
        );
    }
    #[test]
    fn events_use_verified_coordinates_and_do_not_round_into_radius() {
        let make = |id: &str, lat: &str| json!({"id":id,"name":"Event","dates":{"start":{"localDate":"2026-10-20"}},"_embedded":{"venues":[{"name":"Venue","location":{"latitude":lat,"longitude":"-0.1"}}]}});
        let d = json!({"_embedded":{"events":[make("inside","51.572"),make("outside","51.573"),make("missing","")]}});
        let (events, unknown) =
            ticketmaster_events(&d, &location(), 5.0, "2026-10-19", "2026-10-30");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "tm:inside");
        assert_eq!(unknown, 1);
    }
    #[test]
    fn council_holidays_merge_half_term_and_survive_year_boundaries() {
        let d = json!({"authority":{"import_status":"ok"},"academicYears":[{"dates":[{"date":"2026-10-23","event_type":"term_end"},{"date":"2026-10-26","end_date":"2026-10-30","event_type":"half_term_start","label":"Half term"},{"date":"2026-11-02","event_type":"term_start"},{"date":"2026-12-18","event_type":"term_end"},{"date":"2027-01-04","event_type":"term_start"}]}]});
        let out = school_signals(
            &d,
            &location(),
            "2026-10-19".parse().unwrap(),
            "2027-01-05".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(out.signals.len(), 2);
        assert_eq!(out.signals[0].title, "Half term");
        assert_eq!(out.signals[1].end_date.as_deref(), Some("2027-01-03"));
        assert!(out.partial);
    }
    #[test]
    fn source_urls_reject_executable_schemes() {
        assert!(https("javascript:alert(1)").is_none());
        assert!(https("https://example.com").is_some());
    }
}
