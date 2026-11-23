export type DiffSide = 'base' | 'ours' | 'theirs';
export type DiffTone = 'base' | 'same' | 'ours-change' | 'theirs-change' | 'conflict' | 'marker' | 'empty';
export type DiffOperation =
  | { kind: 'match'; baseIndex: number; sideIndex: number }
  | { kind: 'insert'; gap: number; sideIndex: number }
  | { kind: 'delete'; baseIndex: number };

export interface DiffLine {
  number: number;
  text: string;
  tone: DiffTone;
}

export interface DiffRow {
  base: DiffLine;
  ours: DiffLine;
  theirs: DiffLine;
}

export interface SideAlignment {
  matches: Map<number, number>;
  insertions: Map<number, number[]>;
}

export function splitLines(value: string): string[] {
  return value.length === 0 ? [] : value.split(/\r?\n/);
}

export function isConflictMarker(value: string): boolean {
  return value.startsWith('<<<<<<<') || value === '=======' || value.startsWith('>>>>>>>');
}

export function lcsTable(baseLines: string[], sideLines: string[]): number[][] {
  const table = Array.from({ length: baseLines.length + 1 }, () =>
    Array<number>(sideLines.length + 1).fill(0),
  );
  for (let baseIndex = baseLines.length - 1; baseIndex >= 0; baseIndex -= 1) {
    for (let sideIndex = sideLines.length - 1; sideIndex >= 0; sideIndex -= 1) {
      table[baseIndex][sideIndex] =
        baseLines[baseIndex] === sideLines[sideIndex]
          ? table[baseIndex + 1][sideIndex + 1] + 1
          : Math.max(table[baseIndex + 1][sideIndex], table[baseIndex][sideIndex + 1]);
    }
  }
  return table;
}

export function alignSide(baseLines: string[], sideLines: string[]): SideAlignment {
  const table = lcsTable(baseLines, sideLines);
  const operations: DiffOperation[] = [];
  let baseIndex = 0;
  let sideIndex = 0;

  while (baseIndex < baseLines.length || sideIndex < sideLines.length) {
    if (
      baseIndex < baseLines.length &&
      sideIndex < sideLines.length &&
      baseLines[baseIndex] === sideLines[sideIndex]
    ) {
      operations.push({ kind: 'match', baseIndex, sideIndex });
      baseIndex += 1;
      sideIndex += 1;
    } else if (
      sideIndex < sideLines.length &&
      (baseIndex === baseLines.length || table[baseIndex][sideIndex + 1] >= table[baseIndex + 1][sideIndex])
    ) {
      operations.push({ kind: 'insert', gap: baseIndex, sideIndex });
      sideIndex += 1;
    } else {
      operations.push({ kind: 'delete', baseIndex });
      baseIndex += 1;
    }
  }

  const matches = new Map<number, number>();
  const insertions = new Map<number, number[]>();
  let operationIndex = 0;
  while (operationIndex < operations.length) {
    const operation = operations[operationIndex];
    if (operation.kind === 'match') {
      matches.set(operation.baseIndex, operation.sideIndex);
      operationIndex += 1;
      continue;
    }

    const deletes: Extract<DiffOperation, { kind: 'delete' }>[] = [];
    const inserts: Extract<DiffOperation, { kind: 'insert' }>[] = [];
    while (operationIndex < operations.length && operations[operationIndex].kind !== 'match') {
      const current = operations[operationIndex];
      if (current.kind === 'delete') deletes.push(current);
      else if (current.kind === 'insert') inserts.push(current);
      operationIndex += 1;
    }

    const pairedCount = Math.min(deletes.length, inserts.length);
    for (let index = 0; index < pairedCount; index += 1) {
      matches.set(deletes[index].baseIndex, inserts[index].sideIndex);
    }
    for (const insert of inserts.slice(pairedCount)) {
      const lines = insertions.get(insert.gap) ?? [];
      lines.push(insert.sideIndex);
      insertions.set(insert.gap, lines);
    }
  }

  return { matches, insertions };
}

export function toneFor(
  side: DiffSide,
  text: string,
  exists: boolean,
  baseText: string | undefined,
  oursText: string | undefined,
  theirsText: string | undefined,
): DiffTone {
  if (!exists) return 'empty';
  if (isConflictMarker(text)) return 'marker';

  const hasConflict =
    oursText !== undefined &&
    theirsText !== undefined &&
    oursText !== theirsText &&
    (baseText === undefined || (oursText !== baseText && theirsText !== baseText));
  if (hasConflict) return 'conflict';
  if (side === 'base') return 'base';
  return text !== baseText ? (side === 'ours' ? 'ours-change' : 'theirs-change') : 'same';
}

export function makeLine(
  side: DiffSide,
  index: number | undefined,
  lines: string[],
  baseText: string | undefined,
  oursText: string | undefined,
  theirsText: string | undefined,
): DiffLine {
  const exists = index !== undefined;
  const text = exists ? lines[index] : '';
  return {
    number: exists ? index + 1 : 0,
    text,
    tone: toneFor(side, text, exists, baseText, oursText, theirsText),
  };
}

export function buildDiffRows(base: string, ours: string, theirs: string): DiffRow[] {
  const baseLines = splitLines(base);
  const oursLines = splitLines(ours);
  const theirsLines = splitLines(theirs);
  const oursAlignment = alignSide(baseLines, oursLines);
  const theirsAlignment = alignSide(baseLines, theirsLines);
  const alignedRows: DiffRow[] = [];

  const addRow = (baseIndex: number | undefined, oursIndex: number | undefined, theirsIndex: number | undefined) => {
    const baseText = baseIndex === undefined ? undefined : baseLines[baseIndex];
    const oursText = oursIndex === undefined ? undefined : oursLines[oursIndex];
    const theirsText = theirsIndex === undefined ? undefined : theirsLines[theirsIndex];
    alignedRows.push({
      base: makeLine('base', baseIndex, baseLines, baseText, oursText, theirsText),
      ours: makeLine('ours', oursIndex, oursLines, baseText, oursText, theirsText),
      theirs: makeLine('theirs', theirsIndex, theirsLines, baseText, oursText, theirsText),
    });
  };

  for (let gap = 0; gap <= baseLines.length; gap += 1) {
    const oursInserted = oursAlignment.insertions.get(gap) ?? [];
    const theirsInserted = theirsAlignment.insertions.get(gap) ?? [];
    const insertedCount = Math.max(oursInserted.length, theirsInserted.length);
    for (let index = 0; index < insertedCount; index += 1) {
      addRow(undefined, oursInserted[index], theirsInserted[index]);
    }

    if (gap < baseLines.length) {
      addRow(gap, oursAlignment.matches.get(gap), theirsAlignment.matches.get(gap));
    }
  }

  return alignedRows;
}
