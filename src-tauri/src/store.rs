use crate::model::Settings;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Store {
    path: PathBuf,
}
impl Store {
    pub fn new(dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(dir).map_err(|_| "Could not create the app data folder.")?;
        let store = Self {
            path: dir.join("local-outlook.sqlite3"),
        };
        let db = store.open()?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS cache (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL);").map_err(|_| "Could not initialise local storage.")?;
        Ok(store)
    }
    fn open(&self) -> Result<Connection, String> {
        let db = Connection::open(&self.path).map_err(|_| "Could not open local storage.")?;
        db.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "Local storage is busy.")?;
        Ok(db)
    }
    pub fn settings(&self) -> Result<Settings, String> {
        let value: Option<String> = self
            .open()?
            .query_row("SELECT value FROM settings WHERE id=1", [], |r| r.get(0))
            .optional()
            .map_err(|_| "Could not read settings.")?;
        value
            .map(|v| {
                serde_json::from_str(&v).map_err(|_| {
                    "Saved settings are unreadable. Your cache has been preserved.".into()
                })
            })
            .unwrap_or_else(|| Ok(Settings::default()))
    }
    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|_| "Could not encode settings.")?;
        self.open()?.execute("INSERT INTO settings(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value", [json]).map_err(|_| "Could not save settings.")?;
        Ok(())
    }
    pub fn get<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<(T, DateTime<Utc>)>, String> {
        let row: Option<(String, String)> = self
            .open()?
            .query_row(
                "SELECT value,updated_at FROM cache WHERE key=?1",
                [key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|_| "Could not read the cache.")?;
        // A corrupt cache entry can be replaced on demand; settings are never silently reset.
        Ok(row.and_then(|(v, t)| Some((serde_json::from_str(&v).ok()?, t.parse().ok()?))))
    }
    pub fn put<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        self.put_at(key, value, Utc::now())
    }
    fn put_at<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        time: DateTime<Utc>,
    ) -> Result<(), String> {
        let value = serde_json::to_string(value).map_err(|_| "Could not encode cached data.")?;
        self.open()?.execute("INSERT INTO cache(key,value,updated_at) VALUES(?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at", params![key,value,time.to_rfc3339()]).map_err(|_| "Could not save the cache.")?;
        Ok(())
    }
    pub fn clear_cache(&self) -> Result<(), String> {
        self.open()?
            .execute("DELETE FROM cache", [])
            .map_err(|_| "Could not clear cached data.")?;
        Ok(())
    }
    pub fn prune(&self) -> Result<(), String> {
        let cutoff = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
        self.open()?
            .execute(
                "DELETE FROM cache WHERE updated_at < ?1 AND key != 'credential-status'",
                [cutoff],
            )
            .map_err(|_| "Could not remove expired cache entries.")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_is_persistent_and_clear_preserves_settings() {
        let dir = std::env::temp_dir().join(format!(
            "local-outlook-store-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        let s = Store::new(&dir).unwrap();
        let settings = Settings {
            radius_miles: 12.0,
            ..Settings::default()
        };
        s.save_settings(&settings).unwrap();
        s.put("weather", &vec![1, 2, 3]).unwrap();
        assert_eq!(
            Store::new(&dir)
                .unwrap()
                .get::<Vec<i32>>("weather")
                .unwrap()
                .unwrap()
                .0,
            vec![1, 2, 3]
        );
        s.clear_cache().unwrap();
        assert!(s.get::<Vec<i32>>("weather").unwrap().is_none());
        assert_eq!(s.settings().unwrap().radius_miles, 12.0);
        drop(s);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
