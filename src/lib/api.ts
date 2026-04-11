import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  ImportResult,
  PlaybackInfo,
  TagItem,
  UpdateAppSettingsPayload,
  UpdateVideoPayload,
  VideoFilter,
  VideoItem,
} from "./types";

export function listVideos(filter?: VideoFilter) {
  return invoke<VideoItem[]>("list_videos", {
    filter: filter ?? null,
  });
}

export function importVideos(paths: string[]) {
  return invoke<ImportResult>("import_videos", { paths });
}

export function getVideoDetail(id: string) {
  return invoke<VideoItem>("get_video_detail", { id });
}

export function updateVideoMeta(id: string, payload: UpdateVideoPayload) {
  return invoke<VideoItem>("update_video_meta", { id, payload });
}

export function deleteVideo(id: string) {
  return invoke<void>("delete_video", { id });
}

export function createTag(name: string) {
  return invoke<TagItem>("create_tag", { name });
}

export function listTags() {
  return invoke<TagItem[]>("list_tags");
}

export function deleteTag(id: string) {
  return invoke<void>("delete_tag", { id });
}

export function setVideoTags(id: string, tagIds: string[]) {
  return invoke<VideoItem>("set_video_tags", { id, tagIds });
}

export function getPlaybackSource(id: string) {
  return invoke<PlaybackInfo>("get_playback_source", { id });
}

export function exportVideo(id: string, destinationPath: string) {
  return invoke<void>("export_video", { id, destinationPath });
}

export function readClipboardVideoPaths() {
  return invoke<string[]>("read_clipboard_video_paths");
}

export function getAppSettings() {
  return invoke<AppSettings>("get_app_settings");
}

export function updateAppSettings(payload: UpdateAppSettingsPayload) {
  return invoke<AppSettings>("update_app_settings", { payload });
}

export function setLockPassword(newPassword: string) {
  return invoke<AppSettings>("set_lock_password", { newPassword });
}

export function changeLockPassword(currentPassword: string, newPassword: string) {
  return invoke<AppSettings>("change_lock_password", { currentPassword, newPassword });
}

export function disableLockPassword(password: string) {
  return invoke<AppSettings>("disable_lock_password", { password });
}

export function verifyLockPassword(password: string) {
  return invoke<void>("verify_lock_password", { password });
}
