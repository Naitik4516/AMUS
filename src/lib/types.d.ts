export interface Artist {
  id: number;
  name: string;
  profile_image?: string;
  banner_image?: string;
}

export interface Album {
  id: number;
  name: string;
  cover_art?: string;
  album_artist?: Artist[];
  year?: number;
}

export interface Playlist {
  coverArts: string[];
  id: number;
  name: string;
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
  track_number?: number;
}

export interface GlobalSearchResult {
  result_type: "track" | "artist" | "album" | "playlist";
  score: number;
  track?: Track;
  artist?: Artist;
  album?: Album;
  playlist?: Playlist;
}

export interface TrackDetails extends Track {
  path: string;
  mtime: number;
  play_count: number;
  last_played_at?: string;
  skipped_count: number;
  last_skipped_at?: string;
  year: number;
}

export type SortBy =
  "title" | "artist" | "album" | "duration" | "recently_added";

export type RepeatMode = "OFF" | "ALL" | "ONE";

export type PlaybackSource =
  | { type: "Album"; id: number }
  | { type: "Playlist"; id: number }
  | { type: "Artist"; id: number }
  | { type: "Favorites" }
  | { type: "Search" }
  | { type: "Direct" }
  | { type: "Queue" }
  | { type: "Other" };

export type Context =
  | { type: "Playlist"; id: number; name: string; coverArt: string | null }
  | { type: "Album"; id: number; name: string; coverArt: string | null }
  | {
      type: "Artist";
      id: number;
      name: string;
      profileImage: string | null;
      bannerImage: string | null;
    }
  | { type: "Favorites"; name: "Favorites" };
