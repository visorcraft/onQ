export interface PromptSummary {
  id: string;
  title: string;
  folder: string | null;
  tags: string[];
  favorite: boolean;
  locked: boolean;
  updated: string;
  char_count: number;
  /** Short body excerpt for list rows; empty when locked. */
  preview: string;
}

/** Full prompt returned by `read_prompt` (includes body). */
export interface PromptDetail extends PromptSummary {
  body: string;
}
