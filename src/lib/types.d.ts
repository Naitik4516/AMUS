export interface Artist {
  id: number;
  name: string;
  profile_picture?: string;
}

export interface Album {
  id: number;
  name: string;
  cover_art?: string;
}

export interface Genre {
  id: number;
  name: string;
}

export interface Track {
  id: number;
  title: string;
  artists: Artist[];
  album: Album;
  genre: Genre[];
  duration_seconds: number;
  is_favorite: boolean;
  cover_art?: string;
}

export interface TrackDetails extends Track {
  path: string;
  genre: Genre[];
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
