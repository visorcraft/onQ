export interface PromptSummary {
  id: string;
  title: string;
  folder: string | null;
  tags: string[];
  favorite: boolean;
  locked: boolean;
  updated: string;
  char_count: number;
}