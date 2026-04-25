import { defineStore } from "pinia";
import {
  createTag,
  deleteTag,
  deleteVideo,
  getPlaybackSource,
  importVideos,
  listMonthlyPlayStats,
  listTags,
  listVideos,
  recordVideoPlay,
  setVideoTags,
  updateVideoMeta,
} from "../lib/api";
import { resolveLibrarySelection } from "../lib/library-selection";
import { replaceVideoInList } from "../lib/video-updates";
import type { DailyPlayStat, PlaybackInfo, TagItem, VideoFilter, VideoItem } from "../lib/types";

type LibraryState = {
  videos: VideoItem[];
  tags: TagItem[];
  monthlyPlayStats: DailyPlayStat[];
  selectedVideoId: string | null;
  playback: PlaybackInfo | null;
  loading: boolean;
  error: string | null;
};

export const useLibraryStore = defineStore("library", {
  state: (): LibraryState => ({
    videos: [],
    tags: [],
    monthlyPlayStats: [],
    selectedVideoId: null,
    playback: null,
    loading: false,
    error: null,
  }),

  getters: {
    selectedVideo(state): VideoItem | null {
      return state.videos.find((video) => video.id === state.selectedVideoId) ?? null;
    },
  },

  actions: {
    setError(message: unknown) {
      this.error = message instanceof Error ? message.message : String(message);
    },

    clearError() {
      this.error = null;
    },

    async bootstrap() {
      this.loading = true;
      this.clearError();

      try {
        const { year, month } = getCurrentYearMonth();
        const [videos, tags, monthlyPlayStats] = await Promise.all([
          listVideos(),
          listTags(),
          listMonthlyPlayStats(year, month),
        ]);
        this.videos = videos;
        this.tags = tags;
        this.monthlyPlayStats = monthlyPlayStats;
        this.ensureSelection();
      } catch (error) {
        this.setError(error);
      } finally {
        this.loading = false;
      }
    },

    ensureSelection() {
      this.selectedVideoId = resolveLibrarySelection(
        this.selectedVideoId,
        this.videos.map((video) => video.id),
      );
    },

    async refreshVideos(filter?: VideoFilter) {
      this.videos = await listVideos(filter);
      this.ensureSelection();
    },

    async refreshTags() {
      this.tags = await listTags();
    },

    async refreshMonthlyPlayStats(year: number, month: number) {
      this.monthlyPlayStats = await listMonthlyPlayStats(year, month);
    },

    selectVideo(id: string | null) {
      this.selectedVideoId = id;
    },

    async importPaths(paths: string[]) {
      if (paths.length === 0) {
        return { skipped: [] as string[], importedCount: 0, firstImportedId: null as string | null };
      }

      this.loading = true;
      this.clearError();

      try {
        const result = await importVideos(paths);
        await this.refreshVideos();
        await this.refreshTags();
        return {
          skipped: result.skipped,
          importedCount: result.imported.length,
          firstImportedId: result.imported[0]?.id ?? null,
        };
      } catch (error) {
        this.setError(error);
        return { skipped: [] as string[], importedCount: 0, firstImportedId: null as string | null };
      } finally {
        this.loading = false;
      }
    },

    async saveVideoMeta(id: string, title: string, isFavorite: boolean) {
      this.clearError();
      try {
        const updated = await updateVideoMeta(id, { title, isFavorite });
        this.videos = replaceVideoInList(this.videos, updated);
        if (this.playback?.id === id) {
          this.playback = {
            ...this.playback,
            title: updated.title,
            fileName: updated.fileName,
            filePath: updated.storedPath,
          };
        }
      } catch (error) {
        this.setError(error);
      }
    },

    async toggleFavorite(id: string, nextValue: boolean) {
      this.clearError();
      try {
        const updated = await updateVideoMeta(id, { isFavorite: nextValue });
        this.videos = replaceVideoInList(this.videos, updated);
      } catch (error) {
        this.setError(error);
      }
    },

    async removeVideo(id: string) {
      this.clearError();
      try {
        await deleteVideo(id);
        if (this.playback?.id === id) {
          this.playback = null;
        }
        await this.refreshVideos();
      } catch (error) {
        this.setError(error);
      }
    },

    async addTag(name: string) {
      this.clearError();
      try {
        const created = await createTag(name);
        await this.refreshTags();
        return created;
      } catch (error) {
        this.setError(error);
        return null;
      }
    },

    async removeTag(id: string) {
      this.clearError();
      try {
        await deleteTag(id);
        await this.refreshTags();
        await this.refreshVideos();
        return true;
      } catch (error) {
        this.setError(error);
        return false;
      }
    },

    async updateVideoTags(id: string, tagIds: string[]) {
      this.clearError();
      try {
        const updated = await setVideoTags(id, tagIds);
        this.videos = replaceVideoInList(this.videos, updated);
      } catch (error) {
        this.setError(error);
      }
    },

    async loadPlayback(id: string) {
      this.clearError();
      try {
        this.playback = await getPlaybackSource(id);
        this.selectedVideoId = id;
      } catch (error) {
        this.setError(error);
      }
    },

    async recordPlayback(id: string) {
      this.clearError();
      try {
        const updated = await recordVideoPlay(id);
        this.videos = replaceVideoInList(this.videos, updated);
      } catch (error) {
        this.setError(error);
      }
    },
  },
});

function getCurrentYearMonth() {
  const now = new Date();
  return {
    year: now.getFullYear(),
    month: now.getMonth() + 1,
  };
}
