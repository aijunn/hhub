use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoListItem {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub stored_path: String,
    pub mime_type: String,
    pub file_size: u64,
    pub duration_ms: Option<u64>,
    pub is_favorite: bool,
    pub tag_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagItem {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackInfo {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFilter {
    pub query: Option<String>,
    pub favorite_only: Option<bool>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVideoPayload {
    pub title: Option<String>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: Vec<VideoListItem>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub lock_enabled: bool,
    pub pause_on_blur: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppSettingsPayload {
    pub lock_enabled: Option<bool>,
    pub pause_on_blur: Option<bool>,
}

#[derive(Debug)]
pub struct MediaLibrary {
    root_dir: PathBuf,
}

#[derive(Debug)]
struct StoredVideoRow {
    id: String,
    title: String,
    file_name: String,
    stored_path: String,
    mime_type: String,
    file_size: u64,
    duration_ms: Option<u64>,
    is_favorite: bool,
    created_at: i64,
    updated_at: i64,
}

impl MediaLibrary {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(self.videos_dir()).map_err(|err| err.to_string())?;

        let connection = self.open_connection()?;
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS videos (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    file_name TEXT NOT NULL,
                    stored_path TEXT NOT NULL,
                    mime_type TEXT NOT NULL,
                    file_size INTEGER NOT NULL,
                    duration_ms INTEGER,
                    is_favorite INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS tags (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    created_at INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS video_tags (
                    video_id TEXT NOT NULL,
                    tag_id TEXT NOT NULL,
                    PRIMARY KEY (video_id, tag_id),
                    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS app_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    pause_on_blur INTEGER NOT NULL DEFAULT 0,
                    lock_enabled INTEGER NOT NULL DEFAULT 0,
                    password_hash TEXT,
                    password_salt TEXT,
                    updated_at INTEGER NOT NULL
                );

                INSERT OR IGNORE INTO app_settings (id, pause_on_blur, lock_enabled, password_hash, password_salt, updated_at)
                VALUES (1, 0, 0, NULL, NULL, 0);
                ",
            )
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn import_videos(&self, source_paths: &[String]) -> Result<ImportResult, String> {
        let mut imported = Vec::new();
        let mut skipped = Vec::new();

        for source_path in source_paths {
            match self.import_video(Path::new(source_path)) {
                Ok(video) => imported.push(video),
                Err(_) => skipped.push(source_path.clone()),
            }
        }

        Ok(ImportResult { imported, skipped })
    }

    pub fn import_video(&self, source_path: &Path) -> Result<VideoListItem, String> {
        if !source_path.exists() {
            return Err(format!(
                "source file does not exist: {}",
                source_path.display()
            ));
        }

        let id = Uuid::new_v4().to_string();
        let title = source_path
            .file_stem()
            .and_then(OsStr::to_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Untitled Video")
            .trim()
            .to_string();
        let extension = source_path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_string();
        let file_name = build_file_name(&id, &title, extension.as_str());
        let stored_path = self.videos_dir().join(&file_name);

        fs::copy(source_path, &stored_path).map_err(|err| err.to_string())?;

        let metadata = fs::metadata(&stored_path).map_err(|err| err.to_string())?;
        let now = now_ts();
        let mime_type = infer_mime(extension.as_str()).to_string();

        let connection = self.open_connection()?;
        connection
            .execute(
                "
                INSERT INTO videos (
                    id, title, file_name, stored_path, mime_type, file_size, duration_ms, is_favorite, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 0, ?7, ?8)
                ",
                params![
                    id,
                    title,
                    file_name,
                    stored_path.to_string_lossy().to_string(),
                    mime_type,
                    metadata.len() as i64,
                    now,
                    now
                ],
            )
            .map_err(|err| err.to_string())?;

        self.get_video(&connection, &id)
    }

    pub fn list_videos(&self, filter: Option<VideoFilter>) -> Result<Vec<VideoListItem>, String> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "
                SELECT id, title, file_name, stored_path, mime_type, file_size, duration_ms, is_favorite, created_at, updated_at
                FROM videos
                ORDER BY updated_at DESC, title COLLATE NOCASE ASC
                ",
            )
            .map_err(|err| err.to_string())?;

        let rows = statement
            .query_map([], |row| map_video_row(row))
            .map_err(|err| err.to_string())?;

        let mut videos = Vec::new();
        for row in rows {
            let stored = row.map_err(|err| err.to_string())?;
            videos.push(self.inflate_video(&connection, stored)?);
        }

        Ok(apply_filter(videos, filter))
    }

    pub fn get_video_detail(&self, video_id: &str) -> Result<VideoListItem, String> {
        let connection = self.open_connection()?;
        self.get_video(&connection, video_id)
    }

    pub fn update_video_meta(
        &self,
        video_id: &str,
        payload: UpdateVideoPayload,
    ) -> Result<VideoListItem, String> {
        let connection = self.open_connection()?;
        let current = self.load_video_row(&connection, video_id)?;

        let mut next_title = current.title.clone();
        let mut next_file_name = current.file_name.clone();
        let mut next_stored_path = PathBuf::from(&current.stored_path);

        if let Some(title) = payload.title.filter(|value| !value.trim().is_empty()) {
            let extension = Path::new(&current.file_name)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("");
            next_title = title.trim().to_string();
            next_file_name = build_file_name(video_id, &next_title, extension);
            let renamed_path = self.videos_dir().join(&next_file_name);
            if renamed_path != next_stored_path {
                fs::rename(&next_stored_path, &renamed_path).map_err(|err| err.to_string())?;
                next_stored_path = renamed_path;
            }
        }

        let favorite_flag = payload.is_favorite.unwrap_or(current.is_favorite);
        let updated_at = now_ts();

        connection
            .execute(
                "
                UPDATE videos
                SET title = ?2, file_name = ?3, stored_path = ?4, is_favorite = ?5, updated_at = ?6
                WHERE id = ?1
                ",
                params![
                    video_id,
                    next_title,
                    next_file_name,
                    next_stored_path.to_string_lossy().to_string(),
                    favorite_flag as i64,
                    updated_at
                ],
            )
            .map_err(|err| err.to_string())?;

        self.get_video(&connection, video_id)
    }

    pub fn delete_video(&self, video_id: &str) -> Result<(), String> {
        let connection = self.open_connection()?;
        let current = self.load_video_row(&connection, video_id)?;

        connection
            .execute(
                "DELETE FROM video_tags WHERE video_id = ?1",
                params![video_id],
            )
            .map_err(|err| err.to_string())?;
        connection
            .execute("DELETE FROM videos WHERE id = ?1", params![video_id])
            .map_err(|err| err.to_string())?;

        let stored_path = PathBuf::from(current.stored_path);
        if stored_path.exists() {
            fs::remove_file(stored_path).map_err(|err| err.to_string())?;
        }

        Ok(())
    }

    pub fn create_tag(&self, name: &str) -> Result<TagItem, String> {
        let normalized = name.trim();
        if normalized.is_empty() {
            return Err("tag name cannot be empty".into());
        }

        let connection = self.open_connection()?;
        let now = now_ts();
        let id = Uuid::new_v4().to_string();

        connection
            .execute(
                "INSERT OR IGNORE INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, normalized, now],
            )
            .map_err(|err| err.to_string())?;

        connection
            .query_row(
                "SELECT id, name, created_at FROM tags WHERE name = ?1",
                params![normalized],
                |row| {
                    Ok(TagItem {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .map_err(|err| err.to_string())
    }

    pub fn list_tags(&self) -> Result<Vec<TagItem>, String> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare("SELECT id, name, created_at FROM tags ORDER BY name COLLATE NOCASE ASC")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(TagItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|err| err.to_string())?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(row.map_err(|err| err.to_string())?);
        }

        Ok(tags)
    }

    pub fn delete_tag(&self, tag_id: &str) -> Result<(), String> {
        let connection = self.open_connection()?;

        let deleted = connection
            .execute("DELETE FROM tags WHERE id = ?1", params![tag_id])
            .map_err(|err| err.to_string())?;

        if deleted == 0 {
            return Err("tag not found".into());
        }

        Ok(())
    }

    pub fn set_video_tags(
        &self,
        video_id: &str,
        tag_ids: &[String],
    ) -> Result<VideoListItem, String> {
        let connection = self.open_connection()?;
        self.load_video_row(&connection, video_id)?;

        connection
            .execute(
                "DELETE FROM video_tags WHERE video_id = ?1",
                params![video_id],
            )
            .map_err(|err| err.to_string())?;

        for tag_id in tag_ids {
            connection
                .execute(
                    "INSERT INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                    params![video_id, tag_id],
                )
                .map_err(|err| err.to_string())?;
        }

        self.get_video(&connection, video_id)
    }

    pub fn get_playback_info(&self, video_id: &str) -> Result<PlaybackInfo, String> {
        let connection = self.open_connection()?;
        let row = self.load_video_row(&connection, video_id)?;

        Ok(PlaybackInfo {
            id: row.id,
            title: row.title,
            file_name: row.file_name,
            file_path: row.stored_path,
            mime_type: row.mime_type,
        })
    }

    pub fn export_video(&self, video_id: &str, destination_path: &Path) -> Result<(), String> {
        let connection = self.open_connection()?;
        let row = self.load_video_row(&connection, video_id)?;
        let source_path = PathBuf::from(row.stored_path);

        if !source_path.exists() {
            return Err(format!(
                "source file does not exist: {}",
                source_path.display()
            ));
        }

        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }

        fs::copy(source_path, destination_path).map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn get_app_settings(&self) -> Result<AppSettings, String> {
        let connection = self.open_connection()?;
        self.load_app_settings(&connection)
    }

    pub fn update_app_settings(
        &self,
        payload: UpdateAppSettingsPayload,
    ) -> Result<AppSettings, String> {
        let connection = self.open_connection()?;
        let current = self.load_settings_row(&connection)?;
        let lock_enabled = payload.lock_enabled.unwrap_or(current.lock_enabled);
        let pause_on_blur = payload.pause_on_blur.unwrap_or(current.pause_on_blur);

        if lock_enabled && current.password_hash.is_none() {
            return Err("password lock requires a password".into());
        }

        connection
            .execute(
                "
                UPDATE app_settings
                SET lock_enabled = ?1, pause_on_blur = ?2, updated_at = ?3
                WHERE id = 1
                ",
                params![lock_enabled as i64, pause_on_blur as i64, now_ts()],
            )
            .map_err(|err| err.to_string())?;

        self.load_app_settings(&connection)
    }

    pub fn set_lock_password(&self, new_password: &str) -> Result<AppSettings, String> {
        validate_password(new_password)?;

        let connection = self.open_connection()?;
        let (password_hash, password_salt) = hash_password(new_password)?;
        connection
            .execute(
                "
                UPDATE app_settings
                SET password_hash = ?1, password_salt = ?2, lock_enabled = 1, updated_at = ?3
                WHERE id = 1
                ",
                params![password_hash, password_salt, now_ts()],
            )
            .map_err(|err| err.to_string())?;

        self.load_app_settings(&connection)
    }

    pub fn change_lock_password(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<AppSettings, String> {
        validate_password(new_password)?;

        let connection = self.open_connection()?;
        let current = self.load_settings_row(&connection)?;
        verify_password_against_settings(&current, current_password)?;

        let (password_hash, password_salt) = hash_password(new_password)?;
        connection
            .execute(
                "
                UPDATE app_settings
                SET password_hash = ?1, password_salt = ?2, lock_enabled = 1, updated_at = ?3
                WHERE id = 1
                ",
                params![password_hash, password_salt, now_ts()],
            )
            .map_err(|err| err.to_string())?;

        self.load_app_settings(&connection)
    }

    pub fn disable_lock_password(&self, password: &str) -> Result<AppSettings, String> {
        let connection = self.open_connection()?;
        let current = self.load_settings_row(&connection)?;
        verify_password_against_settings(&current, password)?;

        connection
            .execute(
                "
                UPDATE app_settings
                SET lock_enabled = 0, updated_at = ?1
                WHERE id = 1
                ",
                params![now_ts()],
            )
            .map_err(|err| err.to_string())?;

        self.load_app_settings(&connection)
    }

    pub fn verify_lock_password(&self, password: &str) -> Result<(), String> {
        let connection = self.open_connection()?;
        let current = self.load_settings_row(&connection)?;
        if !current.lock_enabled {
            return Ok(());
        }

        verify_password_against_settings(&current, password)
    }

    fn db_path(&self) -> PathBuf {
        self.root_dir.join("app.db")
    }

    fn videos_dir(&self) -> PathBuf {
        self.root_dir.join("videos")
    }

    fn open_connection(&self) -> Result<Connection, String> {
        Connection::open(self.db_path()).map_err(|err| err.to_string())
    }

    fn get_video(&self, connection: &Connection, video_id: &str) -> Result<VideoListItem, String> {
        let stored = self.load_video_row(connection, video_id)?;
        self.inflate_video(connection, stored)
    }

    fn load_video_row(
        &self,
        connection: &Connection,
        video_id: &str,
    ) -> Result<StoredVideoRow, String> {
        connection
            .query_row(
                "
                SELECT id, title, file_name, stored_path, mime_type, file_size, duration_ms, is_favorite, created_at, updated_at
                FROM videos
                WHERE id = ?1
                ",
                params![video_id],
                map_video_row,
            )
            .optional()
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("video not found: {video_id}"))
    }

    fn inflate_video(
        &self,
        connection: &Connection,
        stored: StoredVideoRow,
    ) -> Result<VideoListItem, String> {
        let tag_ids = self.load_tag_ids(connection, &stored.id)?;

        Ok(VideoListItem {
            id: stored.id,
            title: stored.title,
            file_name: stored.file_name,
            stored_path: stored.stored_path,
            mime_type: stored.mime_type,
            file_size: stored.file_size,
            duration_ms: stored.duration_ms,
            is_favorite: stored.is_favorite,
            tag_ids,
            created_at: stored.created_at,
            updated_at: stored.updated_at,
        })
    }

    fn load_tag_ids(&self, connection: &Connection, video_id: &str) -> Result<Vec<String>, String> {
        let mut statement = connection
            .prepare("SELECT tag_id FROM video_tags WHERE video_id = ?1 ORDER BY rowid ASC")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map(params![video_id], |row| row.get::<_, String>(0))
            .map_err(|err| err.to_string())?;

        let mut tag_ids = Vec::new();
        for row in rows {
            tag_ids.push(row.map_err(|err| err.to_string())?);
        }

        Ok(tag_ids)
    }

    fn load_app_settings(&self, connection: &Connection) -> Result<AppSettings, String> {
        let row = self.load_settings_row(connection)?;
        Ok(AppSettings {
            lock_enabled: row.lock_enabled,
            pause_on_blur: row.pause_on_blur,
        })
    }

    fn load_settings_row(&self, connection: &Connection) -> Result<StoredSettingsRow, String> {
        connection
            .query_row(
                "
                SELECT pause_on_blur, lock_enabled, password_hash, password_salt, updated_at
                FROM app_settings
                WHERE id = 1
                ",
                [],
                |row| {
                    Ok(StoredSettingsRow {
                        pause_on_blur: row.get::<_, i64>(0)? == 1,
                        lock_enabled: row.get::<_, i64>(1)? == 1,
                        password_hash: row.get(2)?,
                        password_salt: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|err| err.to_string())?
            .ok_or_else(|| "settings row not found".to_string())
    }
}

#[derive(Debug)]
struct StoredSettingsRow {
    pause_on_blur: bool,
    lock_enabled: bool,
    password_hash: Option<String>,
    password_salt: Option<String>,
}

fn map_video_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredVideoRow> {
    Ok(StoredVideoRow {
        id: row.get(0)?,
        title: row.get(1)?,
        file_name: row.get(2)?,
        stored_path: row.get(3)?,
        mime_type: row.get(4)?,
        file_size: row.get::<_, i64>(5)? as u64,
        duration_ms: row.get::<_, Option<i64>>(6)?.map(|value| value as u64),
        is_favorite: row.get::<_, i64>(7)? == 1,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn apply_filter(videos: Vec<VideoListItem>, filter: Option<VideoFilter>) -> Vec<VideoListItem> {
    let Some(filter) = filter else {
        return videos;
    };

    let query = filter.query.map(|value| value.to_lowercase());
    let tag_ids = filter.tag_ids.unwrap_or_default();
    let favorite_only = filter.favorite_only.unwrap_or(false);

    videos
        .into_iter()
        .filter(|video| {
            if favorite_only && !video.is_favorite {
                return false;
            }

            if let Some(query) = &query {
                let matches_title = video.title.to_lowercase().contains(query);
                let matches_file = video.file_name.to_lowercase().contains(query);
                if !matches_title && !matches_file {
                    return false;
                }
            }

            if !tag_ids.is_empty()
                && !tag_ids
                    .iter()
                    .all(|tag_id| video.tag_ids.iter().any(|owned| owned == tag_id))
            {
                return false;
            }

            true
        })
        .collect()
}

fn build_file_name(video_id: &str, title: &str, extension: &str) -> String {
    let sanitized_title = sanitize_segment(title);
    let suffix = &video_id[..8];

    if extension.is_empty() {
        format!("{sanitized_title}-{suffix}")
    } else {
        format!("{sanitized_title}-{suffix}.{}", extension.to_lowercase())
    }
}

fn sanitize_segment(input: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for ch in input.chars() {
        let mapped = match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {
                previous_dash = false;
                sanitized.push(ch.to_ascii_lowercase());
                continue;
            }
            _ => '-',
        };

        if !previous_dash {
            sanitized.push(mapped);
            previous_dash = true;
        }
    }

    let trimmed = sanitized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "video".into()
    } else {
        trimmed
    }
}

fn infer_mime(extension: &str) -> &'static str {
    match extension.to_ascii_lowercase().as_str() {
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "m4v" => "video/x-m4v",
        "ogg" | "ogv" => "video/ogg",
        _ => "application/octet-stream",
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time drift")
        .as_secs() as i64
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.trim().len() < 4 {
        return Err("password must be at least 4 characters".into());
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<(String, String), String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| err.to_string())?
        .to_string();
    Ok((hash, salt.as_str().to_string()))
}

fn verify_password_against_settings(
    settings: &StoredSettingsRow,
    password: &str,
) -> Result<(), String> {
    let Some(hash) = &settings.password_hash else {
        return Err("password is not configured".into());
    };
    if settings.password_salt.is_none() {
        return Err("password salt is missing".into());
    }

    let parsed_hash = PasswordHash::new(hash).map_err(|err| err.to_string())?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "incorrect password".to_string())
}

#[cfg(test)]
mod tests {
    use super::{MediaLibrary, UpdateAppSettingsPayload, UpdateVideoPayload};
    use std::{fs, path::PathBuf};
    use tempfile::TempDir;

    fn create_fixture_file(root: &TempDir, name: &str) -> PathBuf {
        let path = root.path().join(name);
        fs::write(&path, b"fixture-video").expect("write fixture");
        path
    }

    #[test]
    fn import_copies_video_into_library_and_lists_it() {
        let temp = TempDir::new().expect("temp dir");
        let source = create_fixture_file(&temp, "sample.mp4");
        let library = MediaLibrary::new(temp.path().join("library"));

        library.init().expect("init library");
        let imported = library.import_video(&source).expect("import video");
        let videos = library.list_videos(None).expect("list videos");

        assert_eq!(videos.len(), 1);
        assert_eq!(imported.id, videos[0].id);
        assert_eq!(videos[0].title, "sample");
        assert!(temp
            .path()
            .join("library/videos")
            .join(&videos[0].file_name)
            .exists());
    }

    #[test]
    fn rename_updates_database_title_and_stored_file_name() {
        let temp = TempDir::new().expect("temp dir");
        let source = create_fixture_file(&temp, "rename-me.mov");
        let library = MediaLibrary::new(temp.path().join("library"));

        library.init().expect("init library");
        let imported = library.import_video(&source).expect("import video");
        let original_path = temp.path().join("library/videos").join(imported.file_name);

        let renamed = library
            .update_video_meta(
                &imported.id,
                UpdateVideoPayload {
                    title: Some("Evening Cut".into()),
                    is_favorite: None,
                },
            )
            .expect("rename video");

        assert_eq!(renamed.title, "Evening Cut");
        assert!(!original_path.exists());
        assert!(temp
            .path()
            .join("library/videos")
            .join(&renamed.file_name)
            .exists());
    }

    #[test]
    fn delete_removes_row_and_stored_file() {
        let temp = TempDir::new().expect("temp dir");
        let source = create_fixture_file(&temp, "trash-me.mp4");
        let library = MediaLibrary::new(temp.path().join("library"));

        library.init().expect("init library");
        let imported = library.import_video(&source).expect("import video");
        let stored_path = temp.path().join("library/videos").join(imported.file_name);

        library.delete_video(&imported.id).expect("delete video");

        assert!(!stored_path.exists());
        assert!(library.list_videos(None).expect("list videos").is_empty());
    }

    #[test]
    fn set_video_tags_persists_and_returns_tag_links() {
        let temp = TempDir::new().expect("temp dir");
        let source = create_fixture_file(&temp, "tagged.mp4");
        let library = MediaLibrary::new(temp.path().join("library"));

        library.init().expect("init library");
        let imported = library.import_video(&source).expect("import video");
        let tag_a = library.create_tag("Travel").expect("create tag");
        let tag_b = library.create_tag("Favorite Scene").expect("create tag");

        let updated = library
            .set_video_tags(&imported.id, &[tag_a.id.clone(), tag_b.id.clone()])
            .expect("set video tags");

        let tags = library.list_tags().expect("list tags");

        assert_eq!(tags.len(), 2);
        assert_eq!(updated.tag_ids, vec![tag_a.id, tag_b.id]);
    }

    #[test]
    fn delete_tag_removes_tag_and_unlinks_all_videos() {
        let temp = TempDir::new().expect("temp dir");
        let source_a = create_fixture_file(&temp, "tagged-a.mp4");
        let source_b = create_fixture_file(&temp, "tagged-b.mp4");
        let library = MediaLibrary::new(temp.path().join("library"));

        library.init().expect("init library");
        let video_a = library.import_video(&source_a).expect("import video a");
        let video_b = library.import_video(&source_b).expect("import video b");
        let tag = library.create_tag("Travel").expect("create tag");

        library
            .set_video_tags(&video_a.id, std::slice::from_ref(&tag.id))
            .expect("tag video a");
        library
            .set_video_tags(&video_b.id, std::slice::from_ref(&tag.id))
            .expect("tag video b");

        library.delete_tag(&tag.id).expect("delete tag");

        let tags = library.list_tags().expect("list tags");
        let videos = library.list_videos(None).expect("list videos");

        assert!(tags.is_empty());
        assert!(videos.iter().all(|video| video.tag_ids.is_empty()));
    }

    #[test]
    fn export_copies_selected_video_to_requested_destination() {
        let temp = TempDir::new().expect("temp dir");
        let source = create_fixture_file(&temp, "export-me.mp4");
        let library = MediaLibrary::new(temp.path().join("library"));
        let export_path = temp.path().join("exports").join("Evening Cut.mp4");

        library.init().expect("init library");
        let imported = library.import_video(&source).expect("import video");
        library
            .update_video_meta(
                &imported.id,
                UpdateVideoPayload {
                    title: Some("Evening Cut".into()),
                    is_favorite: None,
                },
            )
            .expect("rename video");

        library
            .export_video(&imported.id, &export_path)
            .expect("export video");

        assert!(export_path.exists());
        assert_eq!(
            fs::read(export_path).expect("read export"),
            b"fixture-video"
        );
    }

    #[test]
    fn settings_default_to_unlocked_and_no_blur_pause() {
        let temp = TempDir::new().expect("temp dir");
        let library = MediaLibrary::new(temp.path().join("library"));
        library.init().expect("init library");

        let settings = library.get_app_settings().expect("get settings");

        assert!(!settings.lock_enabled);
        assert!(!settings.pause_on_blur);
    }

    #[test]
    fn set_and_verify_lock_password_roundtrip() {
        let temp = TempDir::new().expect("temp dir");
        let library = MediaLibrary::new(temp.path().join("library"));
        library.init().expect("init library");

        let settings = library
            .set_lock_password("1234")
            .expect("set lock password");
        assert!(settings.lock_enabled);
        assert!(library.verify_lock_password("1234").is_ok());
        assert!(library.verify_lock_password("bad").is_err());
    }

    #[test]
    fn cannot_enable_lock_without_password() {
        let temp = TempDir::new().expect("temp dir");
        let library = MediaLibrary::new(temp.path().join("library"));
        library.init().expect("init library");

        let error = library
            .update_app_settings(UpdateAppSettingsPayload {
                lock_enabled: Some(true),
                pause_on_blur: None,
            })
            .expect_err("should reject lock without password");

        assert!(error.contains("password"));
    }

    #[test]
    fn can_update_blur_pause_and_disable_lock() {
        let temp = TempDir::new().expect("temp dir");
        let library = MediaLibrary::new(temp.path().join("library"));
        library.init().expect("init library");
        library
            .set_lock_password("1234")
            .expect("set lock password");

        let settings = library
            .update_app_settings(UpdateAppSettingsPayload {
                lock_enabled: Some(true),
                pause_on_blur: Some(true),
            })
            .expect("update settings");
        assert!(settings.pause_on_blur);
        assert!(settings.lock_enabled);

        let disabled = library
            .disable_lock_password("1234")
            .expect("disable lock");
        assert!(!disabled.lock_enabled);
    }
}
