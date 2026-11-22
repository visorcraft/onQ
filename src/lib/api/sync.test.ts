import { describe, expect, it } from 'vitest';
import {
  parseConflictText,
  renderConflict,
  resolveConflictText,
  type ConflictChoice,
} from './sync';

function choices(...entries: Array<[number, ConflictChoice]>): Map<number, ConflictChoice> {
  return new Map(entries);
}

describe('renderConflict', () => {
  it('drops a hunk whose resolved rendering is empty instead of leaving a blank line', () => {
    const segments = parseConflictText('before\n<<<<<<< ours\n=======\nTheirs\n>>>>>>> theirs\nafter');
    expect(renderConflict(segments, choices([0, 'ours']))).toBe('before\nafter');
  });

  it('vanishes a both-sides-empty hunk and joins neighbors with a single newline', () => {
    const segments = parseConflictText('before\n<<<<<<< ours\n=======\n>>>>>>> theirs\nafter');
    expect(renderConflict(segments, choices([0, 'both']))).toBe('before\nafter');
  });

  it('adds no leading blank line when the hunk resolves to empty at the start', () => {
    const segments = parseConflictText('<<<<<<< ours\n=======\nTheirs\n>>>>>>> theirs\nafter');
    expect(renderConflict(segments, choices([0, 'ours']))).toBe('after');
  });

  it('adds no trailing blank line when the hunk resolves to empty at the end', () => {
    const segments = parseConflictText('before\n<<<<<<< ours\n=======\nTheirs\n>>>>>>> theirs');
    expect(renderConflict(segments, choices([0, 'ours']))).toBe('before');
  });
});

describe('renderConflict round-trip', () => {
  it('round-trips a multi-hunk document with context between hunks', () => {
    const text =
      'ctx1\n<<<<<<< ours\nA\n=======\nB\n>>>>>>> theirs\nmid\n<<<<<<< ours\nC\n=======\nD\n>>>>>>> theirs\nctx2';
    expect(renderConflict(parseConflictText(text))).toBe(text);
  });

  it('round-trips a document with a trailing newline', () => {
    const text = 'before\n<<<<<<< ours\nOurs\n=======\nTheirs\n>>>>>>> theirs\nafter\n';
    expect(renderConflict(parseConflictText(text))).toBe(text);
  });

  it('round-trips a document with no conflicts', () => {
    const text = 'just some text\nmore text\n';
    expect(renderConflict(parseConflictText(text))).toBe(text);
  });

  it('round-trips a document with an empty ours side', () => {
    const text = '<<<<<<< ours\n\n=======\nTheirs\n>>>>>>> theirs';
    expect(renderConflict(parseConflictText(text))).toBe(text);
  });

  it('normalizes the standard empty-ours shape to the blank-line marker form', () => {
    expect(
      renderConflict(parseConflictText('<<<<<<< ours\n=======\nTheirs\n>>>>>>> theirs')),
    ).toBe('<<<<<<< ours\n\n=======\nTheirs\n>>>>>>> theirs');
  });
});

describe('resolveConflictText', () => {
  const text =
    'before\n<<<<<<< ours\nA1\n=======\nB1\n>>>>>>> theirs\nmid\n<<<<<<< ours\nA2\n=======\nB2\n>>>>>>> theirs\nafter';

  it('resolves each hunk independently with ours', () => {
    expect(resolveConflictText(text, 'ours')).toBe('before\nA1\nmid\nA2\nafter');
  });

  it('resolves each hunk independently with theirs', () => {
    expect(resolveConflictText(text, 'theirs')).toBe('before\nB1\nmid\nB2\nafter');
  });

  it('joins non-empty sides with a newline for both', () => {
    expect(resolveConflictText(text, 'both')).toBe('before\nA1\nB1\nmid\nA2\nB2\nafter');
  });
});

describe('parseConflictText malformed input', () => {
  it('keeps an unclosed hunk at EOF as plain context preserving the original lines', () => {
    const text = 'before\n<<<<<<< ours\nA\n=======\nB';
    const segments = parseConflictText(text);
    expect(segments.every((segment) => segment.kind === 'context')).toBe(true);
    expect(renderConflict(segments)).toBe(text);
  });

  it('keeps a hunk missing the separator as plain context preserving the original lines', () => {
    const text = 'before\n<<<<<<< ours\nA\nB';
    const segments = parseConflictText(text);
    expect(segments.every((segment) => segment.kind === 'context')).toBe(true);
    expect(renderConflict(segments)).toBe(text);
  });

  it('does not treat a context line starting with >>>>> as a marker', () => {
    const text = 'before\n>>>>>>> noise\nafter';
    const segments = parseConflictText(text);
    expect(segments).toEqual([{ kind: 'context', text: 'before\n>>>>>>> noise\nafter' }]);
  });
});
