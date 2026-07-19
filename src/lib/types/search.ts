/** Wire shape sent to the `search` Tauri command. */
export interface SearchQuery {
  /** Full-text + semantic query string. */
  text: string;
  /** Restrict to prompts in this folder (bytes-eq on `prompts.folder`). */
  folder?: string | null;
  /** Restrict to prompts that have any of these tags (BitmapIn on `prompts.tags`). */
  tags_any?: string[];
  /** Restrict to favorite prompts. */
  favorite?: boolean | null;
  /** Restrict to locked prompts. */
  locked?: boolean | null;
  /** Inclusive lower bound on `prompts.char_count`. */
  char_min?: number | null;
  /** Inclusive upper bound on `prompts.char_count`. */
  char_max?: number | null;
  /** Maximum number of hits to return. Defaults to 50 on the backend. */
  limit?: number;
}

/** One ranked search result returned by the `search` Tauri command. */
export interface SearchHit {
  /** Prompt primary key (bytes, hex/base64-decoded to string by serde). */
  id: string;
  title: string;
  folder: string | null;
  tags: string[];
  favorite: boolean;
  locked: boolean;
  char_count: number;
  /** Unix epoch seconds of the last update. */
  updated_at: number;
  /** Reciprocal Rank Fusion score (higher = more relevant). */
  rrf_score: number;
}