export interface Artist {
  id: number;
  name: string;
  profile_image?: string;
  cover_image?: string;
}

export interface Album {
  id: number;
  name: string;
  cover_art?: string;
}

export interface Track {
  id: number;
  title: string;
  artists: Artist[];
  album: Album;
  duration_seconds: number;
  is_favorite: boolean;
  cover_art?: string;
  added_at: string;
}

export interface TrackDetails extends Track {
  path: string;
  mtime: number;
  play_count: number;
  last_played_at?: string;
  skipped_count: number;
  last_skipped_at?: string;
}

export type SortBy =
  | "title"
  | "artist"
  | "album"
  | "duration"
  | "recently_added";

export interface Playlist {
  id: number;
  name: string;
}
