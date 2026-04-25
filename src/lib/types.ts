export type VideoItem = {
  id: string;
  title: string;
  fileName: string;
  storedPath: string;
  mimeType: string;
  fileSize: number;
  durationMs: number | null;
  isFavorite: boolean;
  tagIds: string[];
  createdAt: number;
  updatedAt: number;
  playCount: number;
};

export type DailyPlayStat = {
  date: string;
  playCount: number;
};

export type TagItem = {
  id: string;
  name: string;
  createdAt: number;
};

export type VideoFilter = {
  query?: string | null;
  favoriteOnly?: boolean | null;
  tagIds?: string[] | null;
};

export type UpdateVideoPayload = {
  title?: string | null;
  isFavorite?: boolean | null;
};

export type PlaybackInfo = {
  id: string;
  title: string;
  fileName: string;
  filePath: string;
  mimeType: string;
};

export type ImportResult = {
  imported: VideoItem[];
  skipped: string[];
};

export type AppSettings = {
  lockEnabled: boolean;
  pauseOnBlur: boolean;
  lockOnBlur: boolean;
};

export type UpdateAppSettingsPayload = {
  lockEnabled?: boolean | null;
  pauseOnBlur?: boolean | null;
  lockOnBlur?: boolean | null;
};
