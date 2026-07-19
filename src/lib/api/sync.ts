export type ConflictChoice = 'ours' | 'theirs' | 'both';

export interface ContextSegment {
  kind: 'context';
  text: string;
}

export interface ConflictHunk {
  kind: 'hunk';
  id: number;
  ours: string;
  theirs: string;
}

export type ConflictSegment = ContextSegment | ConflictHunk;

/**
 * Split a diffy-style conflict document into context and conflict hunks.
 *
 * The parser intentionally works on lines instead of using a regular
 * expression over the whole document. That keeps multiline hunks, empty
 * sides, and a trailing newline intact while making the result easy for the
 * resolver to render per hunk.
 */
export function parseConflictText(text: string): ConflictSegment[] {
  const lines = text.split('\n');
  const segments: ConflictSegment[] = [];
  let context: string[] = [];
  let ours: string[] = [];
  let theirs: string[] = [];
  let openingMarker = '';
  let hunkId = 0;
  let mode: 'context' | 'ours' | 'theirs' = 'context';

  const flushContext = () => {
    if (context.length > 0) {
      segments.push({ kind: 'context', text: context.join('\n') });
      context = [];
    }
  };

  for (const line of lines) {
    if (mode === 'context' && line.startsWith('<<<<<<<')) {
      flushContext();
      openingMarker = line;
      mode = 'ours';
      ours = [];
      theirs = [];
      continue;
    }

    if (mode === 'ours' && line === '=======') {
      mode = 'theirs';
      continue;
    }

    if (mode === 'theirs' && line.startsWith('>>>>>>>')) {
      segments.push({
        kind: 'hunk',
        id: hunkId,
        ours: ours.join('\n'),
        theirs: theirs.join('\n'),
      });
      hunkId += 1;
      mode = 'context';
      ours = [];
      theirs = [];
      continue;
    }

    if (mode === 'ours') {
      ours.push(line);
    } else if (mode === 'theirs') {
      theirs.push(line);
    } else {
      context.push(line);
    }
  }

  // Preserve malformed or incomplete markers as ordinary text rather than
  // silently dropping the user's content. Valid merge output always reaches
  // the context state above.
  if (mode === 'ours') {
    context.push(openingMarker, ...ours);
  } else if (mode === 'theirs') {
    context.push(openingMarker, ...ours, '=======', ...theirs);
  }
  flushContext();
  return segments;
}

function selectedHunk(hunk: ConflictHunk, choice: ConflictChoice): string {
  if (choice === 'ours') return hunk.ours;
  if (choice === 'theirs') return hunk.theirs;
  return [hunk.ours, hunk.theirs].filter((part) => part.length > 0).join('\n');
}

/** Render parsed segments, leaving unselected hunks as conflict markers. */
export function renderConflict(
  segments: readonly ConflictSegment[],
  choices: ReadonlyMap<number, ConflictChoice> = new Map(),
): string {
  return segments
    .map((segment) => {
      if (segment.kind === 'context') return segment.text;
      const choice = choices.get(segment.id);
      if (choice) return selectedHunk(segment, choice);
      return `<<<<<<< ours\n${segment.ours}\n=======\n${segment.theirs}\n>>>>>>> theirs`;
    })
    .join('\n');
}

/** Resolve every conflict hunk in a diffy-style document with one choice. */
export function resolveConflictText(text: string, choice: ConflictChoice): string {
  const segments = parseConflictText(text);
  const choices = new Map<number, ConflictChoice>();
  for (const segment of segments) {
    if (segment.kind === 'hunk') choices.set(segment.id, choice);
  }
  return renderConflict(segments, choices);
}
