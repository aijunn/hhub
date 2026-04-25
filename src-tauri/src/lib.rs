mod media_library;

use media_library::{
    AppSettings, DailyPlayStat, ImportResult, MediaLibrary, PlaybackInfo, TagItem,
    UpdateAppSettingsPayload, UpdateVideoPayload, VideoFilter, VideoListItem,
};
use std::path::PathBuf;
use tauri::{Manager, State};

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeFileURL};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSString, NSURL};

struct AppState {
    library: MediaLibrary,
}

#[tauri::command]
fn import_videos(paths: Vec<String>, state: State<'_, AppState>) -> Result<ImportResult, String> {
    state.library.import_videos(&paths)
}

#[tauri::command]
fn list_videos(
    filter: Option<VideoFilter>,
    state: State<'_, AppState>,
) -> Result<Vec<VideoListItem>, String> {
    state.library.list_videos(filter)
}

#[tauri::command]
fn get_video_detail(id: String, state: State<'_, AppState>) -> Result<VideoListItem, String> {
    state.library.get_video_detail(&id)
}

#[tauri::command]
fn update_video_meta(
    id: String,
    payload: UpdateVideoPayload,
    state: State<'_, AppState>,
) -> Result<VideoListItem, String> {
    state.library.update_video_meta(&id, payload)
}

#[tauri::command]
fn delete_video(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.library.delete_video(&id)
}

#[tauri::command]
fn create_tag(name: String, state: State<'_, AppState>) -> Result<TagItem, String> {
    state.library.create_tag(&name)
}

#[tauri::command]
fn list_tags(state: State<'_, AppState>) -> Result<Vec<TagItem>, String> {
    state.library.list_tags()
}

#[tauri::command]
fn delete_tag(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.library.delete_tag(&id)
}

#[tauri::command]
fn set_video_tags(
    id: String,
    tag_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<VideoListItem, String> {
    state.library.set_video_tags(&id, &tag_ids)
}

#[tauri::command]
fn get_playback_source(id: String, state: State<'_, AppState>) -> Result<PlaybackInfo, String> {
    state.library.get_playback_info(&id)
}

#[tauri::command]
fn record_video_play(id: String, state: State<'_, AppState>) -> Result<VideoListItem, String> {
    state.library.record_video_play(&id)
}

#[tauri::command]
fn list_monthly_play_stats(
    year: i64,
    month: i64,
    state: State<'_, AppState>,
) -> Result<Vec<DailyPlayStat>, String> {
    state.library.list_monthly_play_stats(year, month)
}

#[tauri::command]
fn export_video(
    id: String,
    destination_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let destination = PathBuf::from(destination_path);
    state.library.export_video(&id, &destination)
}

#[tauri::command]
fn read_clipboard_video_paths() -> Result<Vec<String>, String> {
    read_clipboard_video_paths_impl()
}

#[tauri::command]
fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state.library.get_app_settings()
}

#[tauri::command]
fn update_app_settings(
    payload: UpdateAppSettingsPayload,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.library.update_app_settings(payload)
}

#[tauri::command]
fn set_lock_password(
    new_password: String,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.library.set_lock_password(&new_password)
}

#[tauri::command]
fn change_lock_password(
    current_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state
        .library
        .change_lock_password(&current_password, &new_password)
}

#[tauri::command]
fn disable_lock_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    state.library.disable_lock_password(&password)
}

#[tauri::command]
fn verify_lock_password(password: String, state: State<'_, AppState>) -> Result<(), String> {
    state.library.verify_lock_password(&password)
}

#[cfg(target_os = "macos")]
fn read_clipboard_video_paths_impl() -> Result<Vec<String>, String> {
    let pasteboard = NSPasteboard::generalPasteboard();
    let Some(items) = pasteboard.pasteboardItems() else {
        return Ok(Vec::new());
    };

    let mut paths = Vec::new();
    for index in 0..items.count() {
        let item = items.objectAtIndex(index);
        let file_url_type = unsafe { NSPasteboardTypeFileURL };
        let Some(raw_url) = item.stringForType(file_url_type) else {
            continue;
        };

        let Some(path) = file_path_from_clipboard_url(&raw_url) else {
            continue;
        };

        if !paths.iter().any(|existing| existing == &path) {
            paths.push(path);
        }
    }

    Ok(paths)
}

#[cfg(not(target_os = "macos"))]
fn read_clipboard_video_paths_impl() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

#[cfg(target_os = "macos")]
fn file_path_from_clipboard_url(value: &NSString) -> Option<String> {
    let url = NSURL::URLWithString(value)?;
    if !url.isFileURL() {
        return None;
    }

    url.to_file_path()?.into_os_string().into_string().ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|err| err.to_string())?;
            let library = MediaLibrary::new(app_data_dir.join("library"));
            library.init()?;
            app.manage(AppState { library });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            import_videos,
            list_videos,
            get_video_detail,
            update_video_meta,
            delete_video,
            create_tag,
            list_tags,
            delete_tag,
            set_video_tags,
            get_playback_source,
            record_video_play,
            list_monthly_play_stats,
            export_video,
            read_clipboard_video_paths,
            get_app_settings,
            update_app_settings,
            set_lock_password,
            change_lock_password,
            disable_lock_password,
            verify_lock_password
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use super::file_path_from_clipboard_url;
    #[cfg(target_os = "macos")]
    use objc2_foundation::NSString;

    #[cfg(target_os = "macos")]
    #[test]
    fn parses_file_urls_from_clipboard_items() {
        let value = NSString::from_str("file:///tmp/test-video.mp4");
        let parsed = file_path_from_clipboard_url(&value).expect("path from file url");

        assert_eq!(parsed, "/tmp/test-video.mp4");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn ignores_non_file_urls_from_clipboard_items() {
        let value = NSString::from_str("https://example.com/video.mp4");
        assert_eq!(file_path_from_clipboard_url(&value), None);
    }
}
