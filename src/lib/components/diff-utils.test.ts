import { describe, expect, it } from 'vitest';
import {
  alignSide,
  buildDiffRows,
  isConflictMarker,
  splitLines,
  toneFor,
} from './diff-utils';

describe('splitLines', () => {
  it('returns an empty array for an empty string', () => {
    expect(splitLines('')).toEqual([]);
  });

  it('splits on LF', () => {
    expect(splitLines('a\nb')).toEqual(['a', 'b']);
  });

  it('splits on CRLF without leaving trailing carriage returns', () => {
    expect(splitLines('a\r\nb')).toEqual(['a', 'b']);
  });

  it('keeps a trailing empty line', () => {
    expect(splitLines('a\n')).toEqual(['a', '']);
  });
});

describe('isConflictMarker', () => {
  it('recognizes a start marker', () => {
    expect(isConflictMarker('<<<<<<< HEAD')).toBe(true);
  });

  it('recognizes an exact separator marker', () => {
    expect(isConflictMarker('=======')).toBe(true);
  });

  it('rejects an 8-character separator (exact-match semantics)', () => {
    expect(isConflictMarker('========')).toBe(false);
  });

  it('recognizes an end marker', () => {
    expect(isConflictMarker('>>>>>>> x')).toBe(true);
  });

  it('rejects a marker with leading whitespace', () => {
    expect(isConflictMarker('  <<<<<<<')).toBe(false);
  });
});

describe('alignSide', () => {
  it('matches every line when the arrays are identical', () => {
    const base = ['a', 'b', 'c'];
    const side = ['a', 'b', 'c'];

    const alignment = alignSide(base, side);

    expect(alignment.matches.size).toBe(3);
    expect(alignment.matches.get(0)).toBe(0);
    expect(alignment.matches.get(1)).toBe(1);
    expect(alignment.matches.get(2)).toBe(2);
    expect(alignment.insertions.size).toBe(0);
  });

  it('lands a pure insertion at the start under gap 0', () => {
    const alignment = alignSide(['a', 'b'], ['x', 'a', 'b']);

    expect(alignment.insertions.get(0)).toEqual([0]);
    expect(alignment.matches.get(0)).toBe(1);
    expect(alignment.matches.get(1)).toBe(2);
  });

  it('lands a pure insertion in the middle under the correct gap key', () => {
    const alignment = alignSide(['a', 'b'], ['a', 'x', 'b']);

    expect(alignment.insertions.get(1)).toEqual([1]);
    expect(alignment.matches.get(0)).toBe(0);
    expect(alignment.matches.get(1)).toBe(2);
  });

  it('lands a pure insertion at the end under the last gap key', () => {
    const alignment = alignSide(['a', 'b'], ['a', 'b', 'x']);

    expect(alignment.insertions.get(2)).toEqual([2]);
    expect(alignment.matches.get(0)).toBe(0);
    expect(alignment.matches.get(1)).toBe(1);
  });

  it('leaves matches empty for deleted indexes', () => {
    const alignment = alignSide(['a', 'x', 'b'], ['a', 'b']);

    expect(alignment.matches.get(0)).toBe(0);
    expect(alignment.matches.get(1)).toBeUndefined();
    expect(alignment.matches.get(2)).toBe(1);
    expect(alignment.insertions.size).toBe(0);
  });

  it('pairs a delete with an insert in the same run and leaves extra inserts in insertions', () => {
    const alignment = alignSide(['a', 'x', 'b'], ['a', 'y', 'z', 'b']);

    expect(alignment.matches.get(0)).toBe(0);
    expect(alignment.matches.get(1)).toBe(1);
    expect(alignment.matches.get(2)).toBe(3);
    expect(alignment.insertions.get(1)).toEqual([2]);
  });
});

describe('toneFor', () => {
  it('returns empty when the line does not exist', () => {
    expect(toneFor('ours', '', false, undefined, undefined, undefined)).toBe('empty');
  });

  it('returns marker for a conflict marker line', () => {
    expect(toneFor('ours', '=======', true, '=======', '=======', '=======')).toBe('marker');
  });

  it('returns conflict when both sides differ from each other and both differ from base', () => {
    expect(toneFor('ours', 'X', true, 'b', 'X', 'Y')).toBe('conflict');
    expect(toneFor('theirs', 'Y', true, 'b', 'X', 'Y')).toBe('conflict');
  });

  it('returns same for a line that matches base even when the other side changed', () => {
    expect(toneFor('ours', 'b', true, 'b', 'b', 'Y')).toBe('same');
  });

  it('returns theirs-change for the side that changed when ours matches base', () => {
    expect(toneFor('theirs', 'Y', true, 'b', 'b', 'Y')).toBe('theirs-change');
  });

  it('returns same for a line unchanged from base', () => {
    expect(toneFor('ours', 'b', true, 'b', 'b', 'b')).toBe('same');
    expect(toneFor('base', 'b', true, 'b', 'b', 'b')).toBe('base');
  });

  it('returns the side change tone when base text is undefined', () => {
    expect(toneFor('ours', 'X', true, undefined, 'X', undefined)).toBe('ours-change');
    expect(toneFor('theirs', 'Y', true, undefined, undefined, 'Y')).toBe('theirs-change');
  });
});

describe('buildDiffRows', () => {
  it('returns an empty array for all-empty strings', () => {
    expect(buildDiffRows('', '', '')).toEqual([]);
  });

  it('builds rows with correct numbers and tones for a mixed edit', () => {
    const base = 'a\nb\nc';
    const ours = 'a\nX\nc\nnew';
    const theirs = 'a\nb\nc';

    const rows = buildDiffRows(base, ours, theirs);

    // 3 base lines + 1 inserted row (per-gap max insertions)
    expect(rows).toHaveLength(4);

    // Every side line number is index + 1 when the side exists, 0 when absent.
    const baseLines = base.split('\n');
    const oursLines = ours.split('\n');
    const theirsLines = theirs.split('\n');
    for (const row of rows) {
      if (row.base.number > 0) {
        expect(row.base.text).toBe(baseLines[row.base.number - 1]);
      } else {
        expect(row.base.text).toBe('');
      }
      if (row.ours.number > 0) {
        expect(row.ours.text).toBe(oursLines[row.ours.number - 1]);
      } else {
        expect(row.ours.text).toBe('');
      }
      if (row.theirs.number > 0) {
        expect(row.theirs.text).toBe(theirsLines[row.theirs.number - 1]);
      } else {
        expect(row.theirs.text).toBe('');
      }
    }

    // Line b on ours: theirs did NOT change, so this is not a conflict.
    expect(rows[1].ours.text).toBe('X');
    expect(rows[1].ours.tone).toBe('ours-change');
    expect(rows[1].ours.tone).not.toBe('conflict');
    expect(rows[1].theirs.tone).toBe('same');

    // Inserted "new" row: theirs does not exist on it.
    const inserted = rows[3];
    expect(inserted.base.number).toBe(0);
    expect(inserted.base.tone).toBe('empty');
    expect(inserted.ours.number).toBe(4);
    expect(inserted.ours.text).toBe('new');
    expect(inserted.theirs.number).toBe(0);
    expect(inserted.theirs.tone).toBe('empty');

    // Explicit numbering for the base-line rows.
    expect(rows[0].base.number).toBe(1);
    expect(rows[0].ours.number).toBe(1);
    expect(rows[0].theirs.number).toBe(1);
    expect(rows[1].base.number).toBe(2);
    expect(rows[1].ours.number).toBe(2);
    expect(rows[1].theirs.number).toBe(2);
    expect(rows[2].base.number).toBe(3);
    expect(rows[2].ours.number).toBe(3);
    expect(rows[2].theirs.number).toBe(3);
  });

  it('marks rows conflict only where both sides changed differently', () => {
    const base = 'a\nb\nc';
    const ours = 'a\nX\nc';
    const theirs = 'a\nY\nc';

    const rows = buildDiffRows(base, ours, theirs);

    expect(rows).toHaveLength(3);
    const bRow = rows[1];
    expect(bRow.ours.tone).toBe('conflict');
    expect(bRow.theirs.tone).toBe('conflict');
    expect(bRow.base.tone).toBe('conflict');
    // Unchanged lines stay same/base.
    expect(rows[0].ours.tone).toBe('same');
    expect(rows[2].ours.tone).toBe('same');
  });
});
