<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  FolderOpen,
  Maximize,
  Minimize,
  PanelLeftClose,
  PanelLeftOpen,
  PanelTopClose,
  PanelTopOpen,
  Pause,
  Pencil,
  Play,
  Search,
  Settings2,
  Share,
  SkipBack,
  SkipForward,
  Square,
  Tag,
  Trash2,
  Undo2,
  Upload,
  Volume2,
  X,
} from "lucide-vue-next";
import { storeToRefs } from "pinia";
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import brandIconUrl from "./assets/hhub-brand.svg";
import themeDarkUrl from "./assets/theme-dark.svg";
import themeLightUrl from "./assets/theme-light.svg";
import {
  changeLockPassword,
  disableLockPassword,
  exportVideo,
  getAppSettings,
  readClipboardVideoPaths,
  setLockPassword,
  updateAppSettings,
  verifyLockPassword,
} from "./lib/api";
import { resolvePostImportSelection } from "./lib/library-view-state";
import { getTagTheme } from "./lib/tag-theme";
import type { AppSettings, VideoItem } from "./lib/types";
import { useLibraryStore } from "./stores/library";

const windowApi = getCurrentWindow();
const library = useLibraryStore();
const {
  error,
  loading,
  playback,
  selectedVideo,
  selectedVideoId,
  tags,
  videos,
} = storeToRefs(library);

const SUPPORTED_EXTENSIONS = ["mp4", "mov", "webm", "m4v", "ogv"];
const PREVIEW_WIDTH_KEY = "hhub.preview-pane-width";
const SIDEBAR_COLLAPSED_KEY = "hhub.sidebar-collapsed";
const THEME_PREFERENCE_KEY = "hhub.theme-preference";
const DEFAULT_PREVIEW_WIDTH = 408;
const MIN_PREVIEW_WIDTH = 320;
const MAX_PREVIEW_WIDTH = 900;
const DELETE_UNDO_MS = 5000;

type ThemeMode = "light" | "dark";
type ContextMenuMode = "default" | "rename";
type ActiveView = "library" | "settings";
type PendingDeleteEntry = {
  video: VideoItem;
  timeoutId: number;
  previousSelectedId: string | null;
  previousPlaybackId: string | null;
};

const searchQuery = ref("");
const favoriteOnly = ref(false);
const activeTagFilters = ref<string[]>([]);
const titleDraft = ref("");
const selectedTagIds = ref<string[]>([]);
const volume = ref(0.75);
const playbackRate = ref(1);
const progress = ref(0);
const duration = ref(0);
const isPlaying = ref(false);
const isWindowFullscreen = ref(false);
const isTheaterMode = ref(false);
const isDragImportActive = ref(false);
const isReadingClipboard = ref(false);
const sidebarCollapsed = ref(readStoredBoolean(SIDEBAR_COLLAPSED_KEY));
const themeMode = ref<ThemeMode>(resolveInitialTheme());
const activeView = ref<ActiveView>("library");
const isUnlocked = ref(false);
const settings = ref<AppSettings | null>(null);
const skippedPaths = ref<string[]>([]);
const statusMessage = ref("");
const previewPaneWidth = ref(readPreviewWidth());
const normalVideoRef = ref<HTMLVideoElement | null>(null);
const theaterVideoRef = ref<HTMLVideoElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);
const renameInputRef = ref<HTMLInputElement | null>(null);
const unlockInputRef = ref<HTMLInputElement | null>(null);
const pendingResumeTime = ref<number | null>(null);
const pendingAutoPlay = ref(false);
const playbackIntent = ref<"idle" | "play">("idle");
const unlockPassword = ref("");
const pendingDeleteId = ref<string | null>(null);

const settingsForms = reactive({
  lockToggle: false,
  pauseOnBlur: false,
  lockOnBlur: false,
  newPassword: "",
  confirmPassword: "",
  currentPassword: "",
  disablePassword: "",
  message: "",
});

const contextMenu = reactive({
  open: false,
  x: 0,
  y: 0,
  videoId: null as string | null,
  mode: "default" as ContextMenuMode,
  showTags: false,
  submenuLeft: false,
  draftTagName: "",
  draftTitle: "",
  creatingTag: false,
  renaming: false,
});

const pendingDeletes = reactive<Record<string, PendingDeleteEntry>>({});

let removeDragListener: (() => void) | null = null;
let removePasteListener: (() => void) | null = null;
let removePasteShortcutListener: (() => void) | null = null;
let removeGlobalClick: (() => void) | null = null;
let removeGlobalKeydown: (() => void) | null = null;
let removeFocusChangeListener: (() => void) | null = null;
let resizingCleanup: (() => void) | null = null;

const pendingDeleteBanner = computed(() =>
  pendingDeleteId.value
    ? (pendingDeletes[pendingDeleteId.value] ?? null)
    : null,
);

const visibleVideos = computed(() =>
  videos.value.filter((video) => !pendingDeletes[video.id]),
);

const filteredVideos = computed(() =>
  visibleVideos.value.filter((video) => {
    if (favoriteOnly.value && !video.isFavorite) {
      return false;
    }

    if (
      searchQuery.value.trim() &&
      !`${video.title} ${video.fileName}`
        .toLowerCase()
        .includes(searchQuery.value.trim().toLowerCase())
    ) {
      return false;
    }

    if (
      activeTagFilters.value.length > 0 &&
      !activeTagFilters.value.every((tagId) => video.tagIds.includes(tagId))
    ) {
      return false;
    }

    return true;
  }),
);

const tagNameById = computed(() => {
  const lookup = new Map<string, string>();
  tags.value.forEach((tag) => lookup.set(tag.id, tag.name));
  return lookup;
});

const videoUrl = computed(() =>
  playback.value?.filePath ? convertFileSrc(playback.value.filePath) : "",
);
const selectedIndex = computed(() =>
  filteredVideos.value.findIndex((video) => video.id === selectedVideoId.value),
);
const contextMenuVideo = computed(
  () =>
    visibleVideos.value.find((video) => video.id === contextMenu.videoId) ??
    null,
);
const themeToggleImage = computed(() =>
  themeMode.value === "dark" ? themeLightUrl : themeDarkUrl,
);
const themeToggleLabel = computed(() =>
  themeMode.value === "dark" ? "切换为浅色模式" : "切换为深色模式",
);

watch(
  selectedVideo,
  (video) => {
    titleDraft.value = video?.title ?? "";
    selectedTagIds.value = [...(video?.tagIds ?? [])];
  },
  { immediate: true },
);

watch(
  playback,
  async () => {
    progress.value = 0;
    duration.value = 0;
    isPlaying.value = false;
    pendingResumeTime.value = 0;
    pendingAutoPlay.value = playbackIntent.value === "play";

    await nextTick();
    resetActivePlayer();
    playbackIntent.value = "idle";
  },
  { deep: true },
);

watch(
  settings,
  (nextSettings) => {
    if (!nextSettings) {
      return;
    }
    settingsForms.lockToggle = nextSettings.lockEnabled;
    settingsForms.pauseOnBlur = nextSettings.pauseOnBlur;
    settingsForms.lockOnBlur = nextSettings.lockOnBlur;
  },
  { immediate: true },
);

watch(
  () => settings.value?.lockEnabled && !isUnlocked.value,
  async (shouldFocus) => {
    if (!shouldFocus) {
      return;
    }

    await nextTick();
    unlockInputRef.value?.focus();
    unlockInputRef.value?.select();
  },
  { immediate: true },
);

watch(volume, (nextValue) => {
  const video = getActiveVideoElement();
  if (video) {
    video.volume = nextValue;
  }
});

watch(playbackRate, (nextValue) => {
  const video = getActiveVideoElement();
  if (video) {
    video.playbackRate = nextValue;
  }
});

watch(previewPaneWidth, (value) => {
  localStorage.setItem(PREVIEW_WIDTH_KEY, String(value));
});

watch(sidebarCollapsed, (value) => {
  localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(value));
});

watch(themeMode, (value) => {
  applyTheme(value);
  localStorage.setItem(THEME_PREFERENCE_KEY, value);
});

watch(
  () => contextMenu.mode,
  async (mode) => {
    if (mode !== "rename" || !contextMenu.open) {
      return;
    }

    await nextTick();
    renameInputRef.value?.focus();
    renameInputRef.value?.select();
  },
);

onMounted(async () => {
  applyTheme(themeMode.value);
  await refreshSettings();
  if (!settings.value?.lockEnabled) {
    isUnlocked.value = true;
    await bootstrapLibrary();
  }

  isWindowFullscreen.value = await windowApi.isFullscreen();

  removeDragListener = await windowApi.onDragDropEvent((event) => {
    if (event.payload.type === "over") {
      isDragImportActive.value = true;
      return;
    }

    if (event.payload.type === "drop") {
      isDragImportActive.value = false;
      void importFromPaths(event.payload.paths);
      return;
    }

    isDragImportActive.value = false;
  });

  removeFocusChangeListener = await windowApi.onFocusChanged(
    ({ payload: focused }) => {
      if (!focused && settings.value?.pauseOnBlur) {
        const active = getActiveVideoElement();
        active?.pause();
      }

      if (!focused && settings.value?.lockEnabled && settings.value?.lockOnBlur) {
        isUnlocked.value = false;
        unlockPassword.value = "";
        settingsForms.message = "";
      }
    },
  );

  const handlePaste = (event: ClipboardEvent) => {
    if (isEditableTarget(event.target) || !isUnlocked.value) {
      return;
    }

    event.preventDefault();
    void importClipboardVideos();
  };

  const handlePasteShortcut = (event: KeyboardEvent) => {
    if (isEditableTarget(event.target) || !isUnlocked.value) {
      return;
    }

    if (
      event.metaKey &&
      !event.ctrlKey &&
      !event.altKey &&
      event.key.toLowerCase() === "v"
    ) {
      event.preventDefault();
      void importClipboardVideos();
    }
  };

  const closeMenuOnClick = () => closeContextMenu();
  const closeMenuOnEscape = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      closeContextMenu();
    }
  };

  window.addEventListener("paste", handlePaste);
  window.addEventListener("keydown", handlePasteShortcut);
  window.addEventListener("click", closeMenuOnClick);
  window.addEventListener("keydown", closeMenuOnEscape);

  removePasteListener = () => window.removeEventListener("paste", handlePaste);
  removePasteShortcutListener = () =>
    window.removeEventListener("keydown", handlePasteShortcut);
  removeGlobalClick = () =>
    window.removeEventListener("click", closeMenuOnClick);
  removeGlobalKeydown = () =>
    window.removeEventListener("keydown", closeMenuOnEscape);
});

onBeforeUnmount(() => {
  removeDragListener?.();
  removePasteListener?.();
  removePasteShortcutListener?.();
  removeGlobalClick?.();
  removeGlobalKeydown?.();
  removeFocusChangeListener?.();
  resizingCleanup?.();
  Object.values(pendingDeletes).forEach((entry) =>
    window.clearTimeout(entry.timeoutId),
  );
});

async function bootstrapLibrary() {
  await library.bootstrap();
}

async function refreshSettings() {
  settings.value = await getAppSettings();
}

async function unlockApp() {
  settingsForms.message = "";
  try {
    await verifyLockPassword(unlockPassword.value);
    isUnlocked.value = true;
    unlockPassword.value = "";
    await bootstrapLibrary();
  } catch (cause) {
    settingsForms.message =
      cause instanceof Error ? cause.message : String(cause);
  }
}

async function pickVideos() {
  activeView.value = "library";
  try {
    const selection = await open({
      title: "导入本地视频",
      multiple: true,
      filters: [{ name: "Videos", extensions: SUPPORTED_EXTENSIONS }],
    });

    const paths = Array.isArray(selection)
      ? selection
      : selection
        ? [selection]
        : [];
    await importFromPaths(paths);
  } catch (cause) {
    library.setError(cause);
  }
}

async function importClipboardVideos() {
  if (isReadingClipboard.value) {
    return;
  }

  isReadingClipboard.value = true;
  try {
    const paths = await readClipboardVideoPaths();
    await importFromPaths(paths);
  } catch (cause) {
    library.setError(cause);
  } finally {
    isReadingClipboard.value = false;
  }
}

async function importFromPaths(paths: string[]) {
  const { accepted, rejected } = splitImportPaths(paths);
  if (accepted.length === 0) {
    statusMessage.value = "";
    skippedPaths.value = rejected;
    return;
  }

  const result = await library.importPaths(accepted);
  const nextSelectionId = resolvePostImportSelection({
    firstImportedId: result.firstImportedId,
    selectedVideoId: selectedVideoId.value,
    playbackId: playback.value?.id ?? null,
  });

  if (!selectedVideoId.value && !playback.value?.id && nextSelectionId) {
    await loadVideoForPreview(nextSelectionId);
  } else if (nextSelectionId && nextSelectionId !== selectedVideoId.value) {
    library.selectVideo(nextSelectionId);
  }

  skippedPaths.value = [...rejected, ...result.skipped];
  statusMessage.value =
    result.importedCount > 0 ? `已导入 ${result.importedCount} 个视频` : "";
}

async function selectVideo(video: VideoItem) {
  library.selectVideo(video.id);

  if (playback.value?.id === video.id) {
    return;
  }

  await loadVideoForPreview(video.id);
}

async function playVideo(video: VideoItem) {
  playbackIntent.value = "play";
  await library.loadPlayback(video.id);
}

async function loadVideoForPreview(videoId: string) {
  playbackIntent.value = "idle";
  await library.loadPlayback(videoId);
}

async function saveSelectedVideo() {
  if (!selectedVideo.value) {
    return;
  }

  await library.saveVideoMeta(
    selectedVideo.value.id,
    titleDraft.value,
    selectedVideo.value.isFavorite,
  );
  if (playback.value?.id === selectedVideo.value.id) {
    await refreshPlaybackAfterLibraryMutation(selectedVideo.value.id, false);
  }
  statusMessage.value = "已保存视频信息";
}

async function toggleFavorite(video: VideoItem) {
  await library.toggleFavorite(video.id, !video.isFavorite);
}

function deleteVideoTarget(video: VideoItem) {
  const confirmed = window.confirm(`Delete "${video.title}" from the library?`);
  if (!confirmed) {
    return;
  }

  if (pendingDeletes[video.id]) {
    return;
  }

  const previousSelectedId = selectedVideoId.value;
  const previousPlaybackId = playback.value?.id ?? null;
  const timeoutId = window.setTimeout(async () => {
    delete pendingDeletes[video.id];
    if (pendingDeleteId.value === video.id) {
      pendingDeleteId.value = null;
    }
    await library.removeVideo(video.id);
  }, DELETE_UNDO_MS);

  pendingDeletes[video.id] = {
    video,
    timeoutId,
    previousSelectedId,
    previousPlaybackId,
  };
  pendingDeleteId.value = video.id;
  closeContextMenu();

  if (selectedVideoId.value === video.id) {
    const replacement = visibleVideos.value.find(
      (item) => item.id !== video.id && !pendingDeletes[item.id],
    );
    library.selectVideo(replacement?.id ?? null);
  }
  if (playback.value?.id === video.id) {
    library.playback = null;
  }
}

async function undoDelete() {
  const entry = pendingDeleteBanner.value;
  if (!entry) {
    return;
  }

  window.clearTimeout(entry.timeoutId);
  delete pendingDeletes[entry.video.id];
  if (pendingDeleteId.value === entry.video.id) {
    pendingDeleteId.value = null;
  }
  if (entry.previousSelectedId === entry.video.id) {
    library.selectVideo(entry.video.id);
  }
  if (entry.previousPlaybackId === entry.video.id) {
    await refreshPlaybackAfterLibraryMutation(entry.video.id, false);
  }
}

async function saveSelectedTags() {
  if (!selectedVideo.value) {
    return;
  }

  await library.updateVideoTags(selectedVideo.value.id, selectedTagIds.value);
}

async function toggleTagFromMenu(tagId: string) {
  const video = contextMenuVideo.value;
  if (!video) {
    return;
  }

  const nextTagIds = video.tagIds.includes(tagId)
    ? video.tagIds.filter((value) => value !== tagId)
    : [...video.tagIds, tagId];

  await library.updateVideoTags(video.id, nextTagIds);
}

async function createTagFromContextMenu() {
  const video = contextMenuVideo.value;
  const tagName = contextMenu.draftTagName.trim();
  if (!video || !tagName || contextMenu.creatingTag) {
    return;
  }

  contextMenu.creatingTag = true;
  try {
    const created = await library.addTag(tagName);
    if (!created) {
      return;
    }

    const nextTagIds = video.tagIds.includes(created.id)
      ? video.tagIds
      : [...video.tagIds, created.id];
    await library.updateVideoTags(video.id, nextTagIds);
    contextMenu.draftTagName = "";
    contextMenu.showTags = true;
  } finally {
    contextMenu.creatingTag = false;
  }
}

async function deleteTagEntry(tagId: string) {
  const tag = tags.value.find((item) => item.id === tagId);
  if (!tag) {
    return;
  }

  const confirmed = window.confirm(
    `Delete tag "${tag.name}" from the library?`,
  );
  if (!confirmed) {
    return;
  }

  const removed = await library.removeTag(tagId);
  if (!removed) {
    return;
  }

  activeTagFilters.value = activeTagFilters.value.filter(
    (value) => value !== tagId,
  );
  selectedTagIds.value = selectedTagIds.value.filter(
    (value) => value !== tagId,
  );

  if (contextMenuVideo.value) {
    contextMenu.showTags = true;
  }
}

async function renameContextVideo() {
  const video = contextMenuVideo.value;
  const nextTitle = contextMenu.draftTitle.trim();
  if (!video || !nextTitle || contextMenu.renaming) {
    return;
  }

  contextMenu.renaming = true;
  try {
    await library.saveVideoMeta(video.id, nextTitle, video.isFavorite);
    if (playback.value?.id === video.id) {
      await refreshPlaybackAfterLibraryMutation(video.id, false);
    }
    statusMessage.value = "已重命名视频";
    closeContextMenu();
  } finally {
    contextMenu.renaming = false;
  }
}

async function exportContextVideo() {
  const video = contextMenuVideo.value;
  if (!video) {
    return;
  }

  const extension = video.fileName.split(".").pop() ?? "";
  const fileName = extension ? `${video.title}.${extension}` : video.title;
  const destination = await save({
    title: "导出视频",
    defaultPath: fileName,
    filters: [
      {
        name: "Video",
        extensions: extension ? [extension] : SUPPORTED_EXTENSIONS,
      },
    ],
  });

  if (!destination) {
    return;
  }

  await exportVideo(video.id, destination);
  closeContextMenu();
  statusMessage.value = "已导出视频";
}

function toggleTagFilter(tagId: string) {
  activeView.value = "library";
  activeTagFilters.value = activeTagFilters.value.includes(tagId)
    ? activeTagFilters.value.filter((value) => value !== tagId)
    : [...activeTagFilters.value, tagId];
}

function toggleFavoriteFilter() {
  activeView.value = "library";
  favoriteOnly.value = !favoriteOnly.value;
}

function toggleSelectedTag(tagId: string) {
  selectedTagIds.value = selectedTagIds.value.includes(tagId)
    ? selectedTagIds.value.filter((value) => value !== tagId)
    : [...selectedTagIds.value, tagId];
}

function togglePlayback() {
  const video = getActiveVideoElement();
  if (!video) {
    return;
  }

  if (video.paused) {
    void video.play();
  } else {
    video.pause();
  }
}

function stopPlayback() {
  const video = getActiveVideoElement();
  if (!video) {
    return;
  }

  video.pause();
  video.currentTime = 0;
  progress.value = 0;
}

async function playRelative(offset: -1 | 1) {
  const target = filteredVideos.value[selectedIndex.value + offset];
  if (!target) {
    return;
  }

  await playVideo(target);
}

async function toggleWindowFullscreen() {
  const wasPlaying = Boolean(
    getActiveVideoElement() && !getActiveVideoElement()!.paused,
  );
  if (!isTheaterMode.value) {
    prepareModeTransition(wasPlaying);
    isTheaterMode.value = true;
    await nextTick();
    resetActivePlayer();
  } else {
    prepareModeTransition(wasPlaying);
  }

  await windowApi.setFullscreen(!isWindowFullscreen.value);
  isWindowFullscreen.value = await windowApi.isFullscreen();
}

async function toggleTheaterMode() {
  if (isWindowFullscreen.value && isTheaterMode.value) {
    await windowApi.setFullscreen(false);
    isWindowFullscreen.value = await windowApi.isFullscreen();
  }

  const active = getActiveVideoElement();
  const shouldResume = Boolean(active && !active.paused);
  prepareModeTransition(shouldResume);
  isTheaterMode.value = !isTheaterMode.value;
  closeContextMenu();

  await nextTick();
  resetActivePlayer();
}

function onTimeUpdate() {
  const video = getActiveVideoElement();
  if (!video) {
    return;
  }

  progress.value = video.currentTime;
  duration.value = video.duration || 0;
}

function onSeek(event: Event) {
  const target = event.target as HTMLInputElement;
  const nextValue = Number(target.value);
  progress.value = nextValue;

  const video = getActiveVideoElement();
  if (video) {
    video.currentTime = nextValue;
  }
}

function onLoadedMetadata() {
  const video = getActiveVideoElement();
  if (!video) {
    return;
  }

  duration.value = video.duration || 0;

  if (pendingResumeTime.value !== null) {
    video.currentTime = Math.min(
      pendingResumeTime.value,
      video.duration || pendingResumeTime.value,
    );
    pendingResumeTime.value = null;
  }

  if (pendingAutoPlay.value) {
    void video.play().catch(() => undefined);
    pendingAutoPlay.value = false;
  }

  video.volume = volume.value;
  video.playbackRate = playbackRate.value;
}

function onPlay() {
  isPlaying.value = true;
}

function onPause() {
  isPlaying.value = false;
}

function beginPreviewResize(event: MouseEvent) {
  event.preventDefault();

  const startX = event.clientX;
  const startWidth = previewPaneWidth.value;
  document.body.classList.add("is-resizing");

  const handleMove = (moveEvent: MouseEvent) => {
    const delta = startX - moveEvent.clientX;
    previewPaneWidth.value = clamp(
      startWidth + delta,
      MIN_PREVIEW_WIDTH,
      MAX_PREVIEW_WIDTH,
    );
  };

  const handleUp = () => {
    document.body.classList.remove("is-resizing");
    window.removeEventListener("mousemove", handleMove);
    window.removeEventListener("mouseup", handleUp);
    resizingCleanup = null;
  };

  window.addEventListener("mousemove", handleMove);
  window.addEventListener("mouseup", handleUp);
  resizingCleanup = handleUp;
}

function openVideoContextMenu(event: MouseEvent, video: VideoItem) {
  library.selectVideo(video.id);

  const menuWidth = 230;
  const menuHeight = 220;
  const x = clamp(event.clientX, 12, window.innerWidth - menuWidth - 12);
  const y = clamp(event.clientY, 12, window.innerHeight - menuHeight - 12);

  contextMenu.open = true;
  contextMenu.x = x;
  contextMenu.y = y;
  contextMenu.videoId = video.id;
  contextMenu.mode = "default";
  contextMenu.showTags = false;
  contextMenu.submenuLeft = window.innerWidth - x < 470;
  contextMenu.draftTagName = "";
  contextMenu.draftTitle = video.title;
  contextMenu.creatingTag = false;
  contextMenu.renaming = false;
}

function openRenameMenu() {
  contextMenu.mode = "rename";
  contextMenu.showTags = false;
}

function openTagMenu() {
  contextMenu.showTags = true;
}

function closeTagMenu() {
  contextMenu.showTags = false;
}

function closeContextMenu() {
  contextMenu.open = false;
  contextMenu.videoId = null;
  contextMenu.mode = "default";
  contextMenu.showTags = false;
  contextMenu.draftTagName = "";
  contextMenu.draftTitle = "";
  contextMenu.creatingTag = false;
  contextMenu.renaming = false;
}

function toggleTheme() {
  themeMode.value = themeMode.value === "dark" ? "light" : "dark";
}

async function expandSidebarForSearch() {
  activeView.value = "library";
  sidebarCollapsed.value = false;
  await nextTick();
  searchInputRef.value?.focus();
}

// async function startWindowDrag(event: MouseEvent) {
//   const target = event.target as HTMLElement | null;
//   if (target?.closest(".no-drag")) {
//     return;
//   }

//   await windowApi.startDragging();
// }

async function persistPlaybackSettings() {
  settingsForms.message = "";

  try {
    settings.value = await updateAppSettings({
      pauseOnBlur: settingsForms.pauseOnBlur,
      lockOnBlur: settingsForms.lockOnBlur,
    });
  } catch (cause) {
    settingsForms.pauseOnBlur = settings.value?.pauseOnBlur ?? false;
    settingsForms.lockOnBlur = settings.value?.lockOnBlur ?? false;
    settingsForms.message =
      cause instanceof Error ? cause.message : String(cause);
  }
}

async function togglePauseOnBlurSetting() {
  settingsForms.pauseOnBlur = !settingsForms.pauseOnBlur;
  await persistPlaybackSettings();
}

async function toggleLockOnBlurSetting() {
  if (!settings?.value?.lockEnabled) {
    return;
  }

  settingsForms.lockOnBlur = !settingsForms.lockOnBlur;
  await persistPlaybackSettings();
}

async function enableLock() {
  settingsForms.message = "";
  if (settingsForms.newPassword !== settingsForms.confirmPassword) {
    settingsForms.message = "两次输入的新密码不一致";
    return;
  }

  try {
    settings.value = await setLockPassword(settingsForms.newPassword);
    settingsForms.newPassword = "";
    settingsForms.confirmPassword = "";
    settingsForms.lockToggle = true;
    settingsForms.message = "已启用密码锁";
  } catch (cause) {
    settingsForms.message =
      cause instanceof Error ? cause.message : String(cause);
  }
}

async function changePassword() {
  settingsForms.message = "";
  if (settingsForms.newPassword !== settingsForms.confirmPassword) {
    settingsForms.message = "两次输入的新密码不一致";
    return;
  }

  try {
    settings.value = await changeLockPassword(
      settingsForms.currentPassword,
      settingsForms.newPassword,
    );
    settingsForms.currentPassword = "";
    settingsForms.newPassword = "";
    settingsForms.confirmPassword = "";
    settingsForms.message = "已更新密码";
  } catch (cause) {
    settingsForms.message =
      cause instanceof Error ? cause.message : String(cause);
  }
}

async function disableLock() {
  settingsForms.message = "";
  try {
    settings.value = await disableLockPassword(settingsForms.disablePassword);
    settingsForms.disablePassword = "";
    settingsForms.currentPassword = "";
    settingsForms.newPassword = "";
    settingsForms.confirmPassword = "";
    settingsForms.lockToggle = false;
    settingsForms.message = "已关闭密码锁";
  } catch (cause) {
    settingsForms.message =
      cause instanceof Error ? cause.message : String(cause);
  }
}

function formatDuration(totalSeconds: number) {
  if (!Number.isFinite(totalSeconds) || totalSeconds <= 0) {
    return "00:00";
  }

  const seconds = Math.floor(totalSeconds % 60)
    .toString()
    .padStart(2, "0");
  const minutes = Math.floor((totalSeconds / 60) % 60)
    .toString()
    .padStart(2, "0");
  const hours = Math.floor(totalSeconds / 3600);

  return hours > 0 ? `${hours}:${minutes}:${seconds}` : `${minutes}:${seconds}`;
}

function formatBytes(bytes: number) {
  if (!bytes) {
    return "0 B";
  }

  const units = ["B", "KB", "MB", "GB"];
  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  );
  const value = bytes / 1024 ** exponent;
  return `${value.toFixed(exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

function getVideoTagNames(video: VideoItem) {
  return video.tagIds
    .map((tagId) => tagNameById.value.get(tagId))
    .filter((tagName): tagName is string => Boolean(tagName));
}

function getVisibleTagNames(video: VideoItem) {
  return getVideoTagNames(video).slice(0, 2);
}

function getHiddenTagCount(video: VideoItem) {
  return Math.max(getVideoTagNames(video).length - 2, 0);
}

function getTagStyle(tagName: string) {
  const theme = getTagTheme(tagName);

  return {
    "--tag-bg": theme.background,
    "--tag-border": theme.border,
    "--tag-text": theme.text,
    "--tag-accent": theme.accent,
  };
}

function splitImportPaths(paths: string[]) {
  const seen = new Set<string>();
  const accepted: string[] = [];
  const rejected: string[] = [];

  paths.forEach((path) => {
    const normalized = path.toLowerCase();
    if (seen.has(normalized)) {
      return;
    }
    seen.add(normalized);

    if (
      SUPPORTED_EXTENSIONS.includes(path.split(".").pop()?.toLowerCase() ?? "")
    ) {
      accepted.push(path);
    } else {
      rejected.push(path);
    }
  });

  return { accepted, rejected };
}

function readPreviewWidth() {
  const raw = Number(localStorage.getItem(PREVIEW_WIDTH_KEY));
  if (!Number.isFinite(raw)) {
    return DEFAULT_PREVIEW_WIDTH;
  }
  return clamp(raw, MIN_PREVIEW_WIDTH, MAX_PREVIEW_WIDTH);
}

function readStoredBoolean(key: string) {
  return localStorage.getItem(key) === "true";
}

function resolveInitialTheme(): ThemeMode {
  const stored = localStorage.getItem(THEME_PREFERENCE_KEY);
  if (stored === "light" || stored === "dark") {
    return stored;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(value: ThemeMode) {
  document.documentElement.setAttribute("data-theme", value);
}

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function isEditableTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target.isContentEditable ||
    ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)
  );
}

function getActiveVideoElement() {
  return isTheaterMode.value ? theaterVideoRef.value : normalVideoRef.value;
}

function prepareModeTransition(shouldResume: boolean) {
  const snapshot = capturePlayerSnapshot();
  pendingResumeTime.value = snapshot.time;
  pendingAutoPlay.value = shouldResume;
  snapshot.video?.pause();
}

function capturePlayerSnapshot() {
  const video = getActiveVideoElement();
  return {
    time: video?.currentTime ?? progress.value,
    video,
  };
}

async function refreshPlaybackAfterLibraryMutation(
  videoId: string,
  shouldResume: boolean,
) {
  prepareModeTransition(shouldResume);
  playbackIntent.value = shouldResume ? "play" : "idle";
  await library.loadPlayback(videoId);
}

function resetActivePlayer() {
  const video = getActiveVideoElement();
  if (!video) {
    return;
  }

  video.volume = volume.value;
  video.playbackRate = playbackRate.value;
  video.load();
}
</script>

<template>
  <div
    class="h-screen overflow-hidden bg-[var(--app-bg)] text-[var(--text-primary)]"
    @contextmenu.prevent
  >
    <div
      v-if="settings?.lockEnabled && !isUnlocked"
      class="flex h-full items-center justify-center px-6"
    >
      <div class="lock-shell w-full max-w-[360px]">
        <p class="lock-shell__brand">HHub</p>
        <h1 class="mt-3 text-2xl font-semibold">输入密码解锁</h1>
        <p class="mt-2 text-sm text-[var(--text-secondary)]">
          应用已启用密码锁，输入密码后才可访问媒体库。
        </p>
        <input
          ref="unlockInputRef"
          v-model="unlockPassword"
          type="password"
          class="mac-input mt-5 w-full"
          placeholder="密码"
          @keydown.enter.prevent="unlockApp"
        />
        <button
          class="mac-primary-button mt-4 w-full justify-center"
          @click="unlockApp"
        >
          解锁
        </button>
        <p
          v-if="settingsForms.message"
          class="mt-3 text-sm text-[var(--danger-strong)]"
        >
          {{ settingsForms.message }}
        </p>
      </div>
    </div>

    <div v-else class="relative flex h-full flex-col overflow-hidden">
      <header
        class="mac-toolbar flex h-[44px] shrink-0 items-center border-b border-[var(--hairline)] pl-[76px] pr-4 backdrop-blur-xl"
      >
        <div class="flex items-center gap-2 pl-4" data-tauri-drag-region>
          <img :src="brandIconUrl" alt="" class="h-6 w-6 rounded-md" />
          <div
            class="text-[16px] font-semibold tracking-[0.1em] text-[var(--text-primary)]"
          >
            HHub
          </div>
        </div>
        <div class="toolbar-drag-zone mx-3 flex-1" data-tauri-drag-region />
        <div class="no-drag flex items-center gap-3">
          <span v-if="loading" class="text-[11px] text-[var(--text-muted)]"
            >同步中</span
          >
          <button
            class="theme-toggle-button"
            :title="themeToggleLabel"
            :aria-label="themeToggleLabel"
            @click="toggleTheme"
          >
            <img
              :src="themeToggleImage"
              :alt="themeToggleLabel"
              class="h-5 w-5 rounded-full"
            />
          </button>
        </div>
      </header>

      <div class="relative min-h-0 flex-1 overflow-hidden">
        <div
          v-if="isDragImportActive"
          class="pointer-events-none absolute inset-0 z-40 flex items-center justify-center bg-[color-mix(in_srgb,var(--surface-strong)_72%,transparent)] backdrop-blur-sm"
        >
          <div
            class="rounded-[24px] border border-[var(--accent-soft)] bg-[var(--surface)] px-8 py-7 text-center shadow-[0_24px_60px_rgba(0,0,0,0.12)]"
          >
            <p class="text-sm font-medium text-[var(--text-primary)]">
              松开以导入视频
            </p>
            <p class="mt-2 text-xs text-[var(--text-secondary)]">
              支持 Finder 拖拽或复制后 Cmd+V
            </p>
          </div>
        </div>

        <div
          v-if="isTheaterMode"
          class="theater-surface absolute inset-0 z-30 overflow-hidden"
        >
          <div class="flex h-full min-h-0 flex-col px-5 pb-5 pt-4">
            <div class="flex items-center justify-between px-2 text-[12px]">
              <div class="min-w-0">
                <p class="section-label theater-text-muted">影院模式</p>
                <p class="mt-1 truncate text-sm theater-text-primary">
                  {{ playback?.title ?? "未播放" }}
                </p>
              </div>

              <button
                class="player-icon-button player-icon-button--ghost"
                title="退出影院模式"
                aria-label="退出影院模式"
                @click="toggleTheaterMode"
              >
                <X :size="16" />
              </button>
            </div>

            <div
              class="relative mt-4 min-h-0 flex-1 overflow-hidden rounded-[28px] border border-[var(--theater-hairline)] bg-black/90 shadow-[0_32px_96px_rgba(0,0,0,0.18)]"
            >
              <div
                class="pointer-events-none absolute inset-x-0 top-0 h-28 bg-gradient-to-b from-black/22 to-transparent"
              />
              <div class="h-full w-full bg-black">
                <video
                  ref="theaterVideoRef"
                  class="h-full w-full object-contain"
                  :src="videoUrl"
                  @timeupdate="onTimeUpdate"
                  @loadedmetadata="onLoadedMetadata"
                  @play="onPlay"
                  @pause="onPause"
                  @ended="onPause"
                />
              </div>

              <div class="absolute inset-x-0 bottom-0 p-5">
                <div
                  class="theater-controls rounded-[22px] px-4 py-4 shadow-[0_18px_40px_rgba(0,0,0,0.18)] backdrop-blur-xl"
                >
                  <div
                    class="flex items-center justify-between text-[11px] theater-text-muted"
                  >
                    <span>{{ formatDuration(progress) }}</span>
                    <span>{{ formatDuration(duration) }}</span>
                  </div>
                  <input
                    class="mac-range mac-range--theater mt-3"
                    type="range"
                    min="0"
                    :max="duration || 0"
                    :value="progress"
                    @input="onSeek"
                  />

                  <div class="mt-4 flex flex-wrap items-center gap-3">
                    <button
                      class="player-icon-button player-icon-button--ghost"
                      title="上一条"
                      aria-label="上一条"
                      @click="playRelative(-1)"
                    >
                      <SkipBack :size="18" />
                    </button>
                    <button
                      class="player-icon-button player-icon-button--primary"
                      :title="isPlaying ? '暂停' : '播放'"
                      :aria-label="isPlaying ? '暂停' : '播放'"
                      @click="togglePlayback"
                    >
                      <Pause v-if="isPlaying" :size="18" />
                      <Play v-else :size="18" />
                    </button>
                    <button
                      class="player-icon-button player-icon-button--ghost"
                      title="停止"
                      aria-label="停止"
                      @click="stopPlayback"
                    >
                      <Square :size="18" />
                    </button>
                    <button
                      class="player-icon-button player-icon-button--ghost"
                      title="下一条"
                      aria-label="下一条"
                      @click="playRelative(1)"
                    >
                      <SkipForward :size="18" />
                    </button>
                    <div class="mx-1 h-5 w-px bg-[var(--theater-divider)]" />
                    <button
                      class="player-icon-button player-icon-button--ghost"
                      :title="
                        isWindowFullscreen ? '退出系统全屏' : '进入系统全屏'
                      "
                      :aria-label="
                        isWindowFullscreen ? '退出系统全屏' : '进入系统全屏'
                      "
                      @click="toggleWindowFullscreen"
                    >
                      <Minimize v-if="isWindowFullscreen" :size="18" />
                      <Maximize v-else :size="18" />
                    </button>
                    <button
                      class="player-icon-button player-icon-button--ghost"
                      title="退出影院模式"
                      aria-label="退出影院模式"
                      @click="toggleTheaterMode"
                    >
                      <PanelTopClose :size="18" />
                    </button>
                    <div class="ml-auto flex items-center gap-3">
                      <div
                        class="inline-flex items-center gap-2 theater-text-muted"
                      >
                        <Volume2 :size="15" />
                        <input
                          v-model="volume"
                          class="mac-range mac-range--theater w-28"
                          type="range"
                          min="0"
                          max="1"
                          step="0.05"
                        />
                      </div>
                      <select
                        v-model="playbackRate"
                        class="mac-select theater-rate-select text-sm theater-text-primary"
                      >
                        <option :value="0.5">0.5x</option>
                        <option :value="1">1x</option>
                        <option :value="1.25">1.25x</option>
                        <option :value="1.5">1.5x</option>
                        <option :value="2">2x</option>
                      </select>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex h-full min-h-0">
          <aside
            class="border-r border-[var(--hairline)] bg-[var(--sidebar-bg)] transition-[width] duration-200 overflow-hidden"
            :class="sidebarCollapsed ? 'w-[88px]' : 'w-[286px]'"
          >
            <div class="h-full overflow-auto px-3 py-4">
              <div class="flex min-h-full flex-col gap-3">
                <div class="mac-panel p-2.5">
                  <div
                    :class="
                      sidebarCollapsed
                        ? 'flex flex-col items-center gap-2'
                        : 'flex items-center justify-between gap-2'
                    "
                  >
                    <button
                      v-if="!sidebarCollapsed"
                      class="text-sm text-[var(--text-primary)] flex items-center gap-1"
                      @click="activeView = 'library'"
                    >
                      <img
                        :src="brandIconUrl"
                        alt=""
                        class="h-6 w-6 rounded-md"
                      />
                      <span class="font-semibold">HHub</span>
                    </button>
                    <button
                      class="sidebar-icon-button"
                      :title="sidebarCollapsed ? '展开侧边栏' : '收起侧边栏'"
                      :aria-label="
                        sidebarCollapsed ? '展开侧边栏' : '收起侧边栏'
                      "
                      @click="sidebarCollapsed = !sidebarCollapsed"
                    >
                      <PanelLeftOpen v-if="sidebarCollapsed" :size="16" />
                      <PanelLeftClose v-else :size="16" />
                    </button>
                  </div>
                </div>

                <div class="mac-panel p-2.5">
                  <div
                    :class="
                      sidebarCollapsed
                        ? 'flex flex-col items-center gap-2'
                        : 'space-y-2'
                    "
                  >
                    <template v-if="sidebarCollapsed">
                      <button
                        class="sidebar-icon-button"
                        title="导入视频"
                        aria-label="导入视频"
                        @click="pickVideos"
                      >
                        <Upload :size="16" />
                      </button>
                    </template>
                    <template v-else>
                      <button
                        class="sidebar-action-button"
                        title="导入视频"
                        aria-label="导入视频"
                        @click="pickVideos"
                      >
                        <Upload :size="16" />
                        <span>导入视频</span>
                      </button>
                    </template>

                    <template v-if="sidebarCollapsed">
                      <button
                        class="sidebar-icon-button"
                        title="展开搜索"
                        aria-label="展开搜索"
                        @click="expandSidebarForSearch"
                      >
                        <Search :size="16" />
                      </button>
                    </template>
                    <template v-else>
                      <label class="sidebar-search">
                        <Search :size="14" />
                        <input
                          ref="searchInputRef"
                          v-model="searchQuery"
                          class="sidebar-search__input"
                          placeholder="搜索标题或文件名"
                          @focus="activeView = 'library'"
                        />
                      </label>
                    </template>
                  </div>
                </div>

                <template v-if="!sidebarCollapsed">
                  <section class="mac-panel p-3">
                    <div class="flex items-center justify-between">
                      <p class="section-label">筛选</p>
                      <span class="text-[11px] text-[var(--text-muted)]">{{
                        filteredVideos.length
                      }}</span>
                    </div>

                    <div class="mt-3 flex flex-wrap gap-2">
                      <button
                        class="mac-chip"
                        :class="{ 'is-selected': favoriteOnly }"
                        @click="toggleFavoriteFilter"
                      >
                        仅看收藏
                      </button>
                      <div
                        v-for="tag in tags"
                        :key="tag.id"
                        class="tag-chip-group tag-chip-group--sidebar"
                        :style="getTagStyle(tag.name)"
                      >
                        <button
                          class="mac-chip mac-chip--tag"
                          :class="{
                            'is-selected': activeTagFilters.includes(tag.id),
                          }"
                          @click="toggleTagFilter(tag.id)"
                        >
                          {{ tag.name }}
                        </button>
                        <button
                          class="tag-chip-group__delete"
                          title="删除标签"
                          aria-label="删除标签"
                          @click.stop="deleteTagEntry(tag.id)"
                        >
                          <X :size="12" />
                        </button>
                      </div>
                    </div>
                  </section>
                </template>

                <div
                  class="mt-auto flex"
                  :class="sidebarCollapsed ? 'justify-center' : ''"
                >
                  <div
                    :class="
                      sidebarCollapsed
                        ? 'flex flex-col items-center gap-2'
                        : 'flex w-full flex-col gap-2'
                    "
                  >
                    <button
                      :class="
                        sidebarCollapsed
                          ? 'sidebar-icon-button'
                          : 'sidebar-action-button'
                      "
                      title="资源库"
                      aria-label="资源库"
                      @click="activeView = 'library'"
                    >
                      <FolderOpen :size="16" />
                      <span v-if="!sidebarCollapsed">资源库</span>
                    </button>
                    <button
                      :class="
                        sidebarCollapsed
                          ? 'sidebar-icon-button'
                          : 'sidebar-action-button'
                      "
                      title="设置"
                      aria-label="设置"
                      @click="activeView = 'settings'"
                    >
                      <Settings2 :size="16" />
                      <span v-if="!sidebarCollapsed">设置</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </aside>

          <main class="min-w-0 flex-1 overflow-hidden bg-[var(--content-bg)]">
            <div
              v-if="activeView === 'settings'"
              class="h-full overflow-auto px-6 py-6"
            >
              <div class="mx-auto max-w-[760px] space-y-6">
                <section class="mac-panel p-5">
                  <div class="flex items-center justify-between">
                    <div>
                      <p class="section-label">安全</p>
                      <h2 class="mt-2 text-lg font-semibold">应用密码锁</h2>
                      <p class="mt-2 text-sm text-[var(--text-secondary)]">
                        启用后，应用启动时需要输入密码才能访问媒体库。
                      </p>
                    </div>
                    <button
                      class="mac-chip"
                      :class="{ 'is-selected': settingsForms.lockToggle }"
                    >
                      {{ settings?.lockEnabled ? "已启用" : "未启用" }}
                    </button>
                  </div>

                  <div
                    v-if="!settings?.lockEnabled"
                    class="mt-5 grid gap-3 md:grid-cols-2"
                  >
                    <input
                      v-model="settingsForms.newPassword"
                      type="password"
                      class="mac-input"
                      placeholder="新密码"
                    />
                    <input
                      v-model="settingsForms.confirmPassword"
                      type="password"
                      class="mac-input"
                      placeholder="确认新密码"
                    />
                  </div>
                  <div v-else class="mt-5 grid gap-3 md:grid-cols-2">
                    <input
                      v-model="settingsForms.currentPassword"
                      type="password"
                      class="mac-input"
                      placeholder="当前密码"
                    />
                    <input
                      v-model="settingsForms.newPassword"
                      type="password"
                      class="mac-input"
                      placeholder="新密码"
                    />
                    <input
                      v-model="settingsForms.confirmPassword"
                      type="password"
                      class="mac-input md:col-span-2"
                      placeholder="确认新密码"
                    />
                  </div>

                  <div class="mt-4 flex flex-wrap gap-2">
                    <button
                      v-if="!settings?.lockEnabled"
                      class="mac-primary-button"
                      @click="enableLock"
                    >
                      启用密码锁
                    </button>
                    <button
                      v-else
                      class="mac-primary-button"
                      @click="changePassword"
                    >
                      修改密码
                    </button>
                  </div>

                  <div
                    v-if="settings?.lockEnabled"
                    class="mt-5 border-t border-[var(--hairline)] pt-5"
                  >
                    <p class="text-sm font-medium text-[var(--text-primary)]">
                      关闭密码锁
                    </p>
                    <div class="mt-3 flex gap-3">
                      <input
                        v-model="settingsForms.disablePassword"
                        type="password"
                        class="mac-input min-w-0 flex-1"
                        placeholder="输入当前密码确认关闭"
                      />
                      <button class="mac-danger-button" @click="disableLock">
                        关闭
                      </button>
                    </div>
                  </div>
                </section>

                <section class="mac-panel p-5">
                  <p class="section-label">播放</p>
                  <h2 class="mt-2 text-lg font-semibold">失焦暂停</h2>
                  <p class="mt-2 text-sm text-[var(--text-secondary)]">
                    当应用失去焦点时，自动暂停正在播放的视频。
                  </p>
                  <div class="mt-4 flex items-center gap-3">
                    <button
                      class="mac-chip"
                      :class="{ 'is-selected': settingsForms.pauseOnBlur }"
                      @click="togglePauseOnBlurSetting"
                    >
                      {{ settingsForms.pauseOnBlur ? "已开启" : "已关闭" }}
                    </button>
                  </div>

                  <div
                    class="mt-4 flex items-center justify-between rounded-[16px] border border-[var(--hairline)] bg-[var(--surface)] px-4 py-3"
                  >
                    <div>
                      <p class="text-sm font-medium">失焦时自动锁定</p>
                      <p class="mt-1 text-xs text-[var(--text-secondary)]">
                        应用失去焦点时自动回到解锁界面，需要已启用密码锁。
                      </p>
                    </div>
                    <button
                      class="mac-chip"
                      :class="{ 'is-selected': settingsForms.lockOnBlur }"
                      :disabled="!settings?.lockEnabled"
                      @click="toggleLockOnBlurSetting"
                    >
                      {{ settingsForms.lockOnBlur ? "已开启" : "已关闭" }}
                    </button>
                  </div>
                </section>

                <div v-if="settingsForms.message" class="mac-banner">
                  {{ settingsForms.message }}
                </div>
              </div>
            </div>

            <div v-else class="flex h-full min-w-0">
              <section class="min-w-0 flex-1 overflow-hidden">
                <div class="h-full overflow-auto px-4 py-4">
                  <div class="mb-3 flex items-center justify-between">
                    <div>
                      <p class="section-label">资源库</p>
                      <p class="mt-1 text-sm text-[var(--text-secondary)]">
                        右键视频项可设置标签、重命名、删除或导出
                      </p>
                    </div>
                  </div>

                  <div
                    v-if="pendingDeleteBanner"
                    class="mac-banner mac-banner--warning flex items-center justify-between gap-3"
                  >
                    <span class="mac-banner__message truncate"
                      >已删除 {{ pendingDeleteBanner.video.title }}，5
                      秒内可撤销</span
                    >
                    <button
                      class="inline-flex items-center gap-2 text-sm font-medium text-[var(--accent-strong)]"
                      @click="undoDelete"
                    >
                      <Undo2 :size="14" />
                      撤销
                    </button>
                  </div>
                  <div v-else-if="error" class="mac-banner mac-banner--error">
                    {{ error }}
                  </div>
                  <div v-else-if="statusMessage" class="mac-banner">
                    {{ statusMessage }}
                  </div>
                  <div
                    v-if="skippedPaths.length > 0"
                    class="mac-banner mac-banner--warning"
                  >
                    已跳过 {{ skippedPaths.length }} 个不支持或不可用的文件
                  </div>

                  <div class="mt-3 grid gap-2 pb-4">
                    <button
                      v-for="video in filteredVideos"
                      :key="video.id"
                      class="mac-row"
                      :class="{ 'is-selected': selectedVideoId === video.id }"
                      @click="selectVideo(video)"
                      @dblclick="playVideo(video)"
                      @contextmenu.stop.prevent="
                        openVideoContextMenu($event, video)
                      "
                    >
                      <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-medium">
                          {{ video.title }}
                        </p>
                        <p
                          class="mt-1 truncate text-xs text-[var(--text-muted)]"
                        >
                          {{ video.fileName }}
                        </p>
                        <div
                          v-if="getVideoTagNames(video).length > 0"
                          class="mt-2 flex flex-wrap gap-1.5"
                        >
                          <span
                            v-for="tagName in getVisibleTagNames(video)"
                            :key="tagName"
                            class="video-tag-chip"
                            :style="getTagStyle(tagName)"
                          >
                            {{ tagName }}
                          </span>
                          <span
                            v-if="getHiddenTagCount(video) > 0"
                            class="video-tag-chip"
                          >
                            +{{ getHiddenTagCount(video) }}
                          </span>
                        </div>
                      </div>

                      <div class="mac-row__meta ml-4 flex shrink-0 items-center gap-2">
                        <span class="truncate text-xs text-[var(--text-muted)]">{{
                          formatBytes(video.fileSize)
                        }}</span>
                        <button
                          class="mac-star"
                          :class="{ 'is-active': video.isFavorite }"
                          @click.stop="toggleFavorite(video)"
                        >
                          ★
                        </button>
                      </div>
                    </button>

                    <div
                      v-if="filteredVideos.length === 0"
                      class="mac-empty-state"
                    >
                      <p class="text-sm font-medium">还没有视频</p>
                      <p class="mt-2 text-xs text-[var(--text-secondary)]">
                        通过左侧边栏导入，或直接把视频拖进窗口。
                      </p>
                    </div>
                  </div>
                </div>
              </section>

              <div class="preview-divider" @mousedown="beginPreviewResize" />

              <section
                class="relative border-l border-[var(--hairline)] bg-[var(--surface)] overflow-hidden"
                :style="{ width: `${previewPaneWidth}px` }"
              >
                <div class="flex h-full flex-col overflow-hidden">
                  <div
                    class="shrink-0 border-b border-[var(--hairline)] px-4 py-4"
                  >
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="section-label">预览</p>
                        <p class="mt-1 text-xs text-[var(--text-secondary)]">
                          {{ playback?.title ?? "选中视频后播放" }}
                        </p>
                      </div>
                    </div>
                  </div>

                  <div class="min-h-0 flex-1 overflow-auto px-4 py-4">
                    <div
                      class="overflow-hidden rounded-[18px] border border-[var(--hairline)] bg-black"
                    >
                      <div class="aspect-video bg-black">
                        <video
                          ref="normalVideoRef"
                          class="h-full w-full object-contain"
                          :src="videoUrl"
                          @timeupdate="onTimeUpdate"
                          @loadedmetadata="onLoadedMetadata"
                          @play="onPlay"
                          @pause="onPause"
                          @ended="onPause"
                        />
                      </div>
                    </div>

                    <div class="mt-4 space-y-4">
                      <div
                        class="flex items-center justify-between text-xs text-[var(--text-secondary)]"
                      >
                        <span>{{ formatDuration(progress) }}</span>
                        <span>{{ formatDuration(duration) }}</span>
                      </div>
                      <input
                        class="mac-range"
                        type="range"
                        min="0"
                        :max="duration || 0"
                        :value="progress"
                        @input="onSeek"
                      />

                      <div class="flex flex-wrap items-center gap-3">
                        <button
                          class="player-icon-button"
                          title="上一条"
                          aria-label="上一条"
                          @click="playRelative(-1)"
                        >
                          <SkipBack :size="16" />
                        </button>
                        <button
                          class="player-icon-button player-icon-button--primary"
                          :title="isPlaying ? '暂停' : '播放'"
                          :aria-label="isPlaying ? '暂停' : '播放'"
                          @click="togglePlayback"
                        >
                          <Pause v-if="isPlaying" :size="16" />
                          <Play v-else :size="16" />
                        </button>
                        <button
                          class="player-icon-button"
                          title="停止"
                          aria-label="停止"
                          @click="stopPlayback"
                        >
                          <Square :size="16" />
                        </button>
                        <button
                          class="player-icon-button"
                          title="下一条"
                          aria-label="下一条"
                          @click="playRelative(1)"
                        >
                          <SkipForward :size="16" />
                        </button>
                        <button
                          class="player-icon-button"
                          title="影院模式"
                          aria-label="影院模式"
                          @click="toggleTheaterMode"
                        >
                          <PanelTopOpen :size="16" />
                        </button>
                        <button
                          class="player-icon-button"
                          :title="
                            isWindowFullscreen ? '退出系统全屏' : '进入系统全屏'
                          "
                          :aria-label="
                            isWindowFullscreen ? '退出系统全屏' : '进入系统全屏'
                          "
                          @click="toggleWindowFullscreen"
                        >
                          <Minimize v-if="isWindowFullscreen" :size="16" />
                          <Maximize v-else :size="16" />
                        </button>
                      </div>

                      <div class="grid gap-3 sm:grid-cols-2">
                        <label class="mac-control-card px-4 py-3">
                          <span
                            class="control-label inline-flex items-center gap-2"
                          >
                            <Volume2 :size="14" />
                            音量
                          </span>
                          <input
                            v-model="volume"
                            class="mac-range mt-3"
                            type="range"
                            min="0"
                            max="1"
                            step="0.05"
                          />
                        </label>
                        <label class="mac-control-card px-4 py-3">
                          <span class="control-label">倍速</span>
                          <select
                            v-model="playbackRate"
                            class="mac-select mt-3 w-full"
                          >
                            <option :value="0.5">0.5x</option>
                            <option :value="1">1x</option>
                            <option :value="1.25">1.25x</option>
                            <option :value="1.5">1.5x</option>
                            <option :value="2">2x</option>
                          </select>
                        </label>
                      </div>
                    </div>

                    <div v-if="selectedVideo" class="mt-5 space-y-4 pb-4">
                      <div class="mac-panel p-4">
                        <div class="flex items-start justify-between gap-3">
                          <div class="min-w-0">
                            <p class="section-label">详情</p>
                            <p class="mt-2 truncate text-sm font-medium">
                              {{ selectedVideo.title }}
                            </p>
                          </div>
                          <button
                            class="mac-star"
                            :class="{ 'is-active': selectedVideo.isFavorite }"
                            @click="toggleFavorite(selectedVideo)"
                          >
                            ★
                          </button>
                        </div>

                        <label class="mt-4 block">
                          <span class="control-label">标题</span>
                          <input
                            v-model="titleDraft"
                            class="mac-input mt-2 w-full"
                          />
                        </label>

                        <div class="mt-4">
                          <div class="flex items-center justify-between">
                            <span class="control-label">标签</span>
                            <button
                              class="text-xs text-[var(--accent-strong)]"
                              @click="saveSelectedTags"
                            >
                              保存
                            </button>
                          </div>
                          <div class="mt-3 flex flex-wrap gap-2">
                            <div
                              v-for="tag in tags"
                              :key="tag.id"
                              class="tag-chip-group"
                              :style="getTagStyle(tag.name)"
                            >
                              <button
                                class="mac-chip mac-chip--tag"
                                :class="{
                                  'is-selected': selectedTagIds.includes(
                                    tag.id,
                                  ),
                                }"
                                @click="toggleSelectedTag(tag.id)"
                              >
                                {{ tag.name }}
                              </button>
                              <button
                                class="tag-chip-group__delete"
                                title="删除标签"
                                aria-label="删除标签"
                                @click.stop="deleteTagEntry(tag.id)"
                              >
                                <X :size="12" />
                              </button>
                            </div>
                          </div>
                        </div>

                        <div
                          class="mt-4 grid grid-cols-2 gap-3 text-xs text-[var(--text-secondary)]"
                        >
                          <div class="mac-metric">
                            <span>大小</span>
                            <strong>{{
                              formatBytes(selectedVideo.fileSize)
                            }}</strong>
                          </div>
                          <div class="mac-metric">
                            <span>格式</span>
                            <strong>{{ selectedVideo.mimeType }}</strong>
                          </div>
                        </div>

                        <div class="mt-4 flex flex-wrap gap-2">
                          <button
                            class="mac-primary-button"
                            @click="saveSelectedVideo"
                          >
                            保存改名
                          </button>
                          <button
                            class="mac-secondary-button"
                            @click="loadVideoForPreview(selectedVideo.id)"
                          >
                            加载预览
                          </button>
                          <button
                            class="mac-danger-button"
                            @click="deleteVideoTarget(selectedVideo)"
                          >
                            删除
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </section>
            </div>
          </main>
        </div>

        <div
          v-if="contextMenu.open && contextMenuVideo"
          class="context-menu"
          :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
          @click.stop
        >
          <template v-if="contextMenu.mode === 'rename'">
            <div class="context-menu__form context-menu__form--stack">
              <input
                ref="renameInputRef"
                v-model="contextMenu.draftTitle"
                class="context-menu__input"
                placeholder="视频标题"
                @click.stop
                @keydown.enter.prevent="renameContextVideo"
                @keydown.esc.prevent="closeContextMenu"
              />
              <div class="flex gap-2">
                <button class="context-menu__ghost" @click="closeContextMenu">
                  取消
                </button>
                <button
                  class="context-menu__submit flex-1"
                  :disabled="contextMenu.renaming"
                  @click="renameContextVideo"
                >
                  重命名
                </button>
              </div>
            </div>
          </template>
          <template v-else>
            <button class="context-menu__item" @click="openRenameMenu">
              <span class="inline-flex items-center gap-2">
                <Pencil :size="14" />
                重命名
              </span>
            </button>
            <div
              class="context-submenu-anchor"
              @mouseenter="openTagMenu"
              @mouseleave="closeTagMenu"
            >
              <button class="context-menu__item" @mouseenter="openTagMenu">
                <span class="inline-flex items-center gap-2">
                  <Tag :size="14" />
                  设置标签
                </span>
                <span class="text-[var(--text-muted)]">›</span>
              </button>

              <div
                v-if="contextMenu.showTags"
                class="context-submenu"
                :class="{ 'context-submenu--left': contextMenu.submenuLeft }"
              >
                <div class="context-submenu__scroller">
                  <div v-if="tags.length > 0" class="context-tag-grid">
                    <button
                      v-for="tag in tags"
                      :key="tag.id"
                      class="context-tag-button"
                      :class="{
                        'is-active': contextMenuVideo.tagIds.includes(tag.id),
                      }"
                      :style="getTagStyle(tag.name)"
                      @click="toggleTagFromMenu(tag.id)"
                    >
                      <span class="inline-flex items-center gap-1.5">
                        <Tag :size="12" />
                        <span class="truncate">{{ tag.name }}</span>
                      </span>
                    </button>
                  </div>
                  <div v-if="tags.length === 0" class="context-menu__empty">
                    暂无标签
                  </div>
                </div>

                <div class="context-menu__divider" />
                <div class="context-menu__form">
                  <input
                    v-model="contextMenu.draftTagName"
                    class="context-menu__input"
                    placeholder="新建标签"
                    @click.stop
                    @keydown.enter.prevent="createTagFromContextMenu"
                  />
                  <button
                    class="context-menu__submit"
                    :disabled="contextMenu.creatingTag"
                    @click="createTagFromContextMenu"
                  >
                    新建
                  </button>
                </div>
              </div>
            </div>
            <button class="context-menu__item" @click="exportContextVideo">
              <span class="inline-flex items-center gap-2">
                <Share :size="14" />
                导出
              </span>
            </button>
            <div class="context-menu__divider" />
            <button
              class="context-menu__item context-menu__item--danger"
              @click="deleteVideoTarget(contextMenuVideo)"
            >
              <span class="inline-flex items-center gap-2">
                <Trash2 :size="14" />
                删除
              </span>
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
