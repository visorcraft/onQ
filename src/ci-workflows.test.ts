/**
 * Structural guards for GitHub Actions config that has already bitten us in CI.
 * These read the real committed workflow/baseline files (not fixtures).
 */
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';

// Vitest runs with cwd = package root (onQ repo).
const repoRoot = resolve(process.cwd());

function readRepo(...parts: string[]): string {
  return readFileSync(join(repoRoot, ...parts), 'utf8');
}

const tempDirs: string[] = [];
afterEach(() => {
  while (tempDirs.length) {
    const d = tempDirs.pop();
    if (d) rmSync(d, { recursive: true, force: true });
  }
});

describe('release.yml', () => {
  const yaml = readRepo('.github', 'workflows', 'release.yml');

  it('sets SOURCE_DATE_EPOCH from git commit time, not an empty github field', () => {
    // Empty SOURCE_DATE_EPOCH breaks objc2/cc-rs on macOS.
    expect(yaml).not.toMatch(/SOURCE_DATE_EPOCH:\s*\$\{\{\s*github\.event\.repository\.push_id\s*\}\}/);
    expect(yaml).toMatch(/SOURCE_DATE_EPOCH=\$\(git log -1 --pretty=%ct\)/);
    // Must not inject a blank env value into the build step.
    const buildEnv = yaml.slice(yaml.indexOf('name: Build Tauri app'));
    expect(buildEnv).not.toMatch(/SOURCE_DATE_EPOCH:\s*\$\{\{/);
  });

  it('enables AppImage extract-and-run and skips linuxdeploy strip on Linux', () => {
    expect(yaml).toMatch(/APPIMAGE_EXTRACT_AND_RUN:\s*['"]?1['"]?/);
    expect(yaml).toMatch(/NO_STRIP:\s*['"]?true['"]?/);
  });

  it('does not target unsupported Intel macOS (ort has no prebuilts)', () => {
    // Comment may mention the triple; matrix `target:` must not use it.
    const matrixTargets = [...yaml.matchAll(/target:\s*([a-z0-9_-]+)/g)].map((m) => m[1]);
    expect(matrixTargets).not.toContain('x86_64-apple-darwin');
    expect(matrixTargets).toEqual(
      expect.arrayContaining([
        'aarch64-apple-darwin',
        'x86_64-unknown-linux-gnu',
        'x86_64-pc-windows-msvc',
      ]),
    );
  });

  it('uploads Tauri macOS updater .app.tar.gz artifacts', () => {
    expect(yaml).toMatch(/macos\/\*\.app\.tar\.gz/);
  });

  it('publishes latest.json for the Tauri static updater endpoint', () => {
    // Without this, check-for-updates fails with
    // "Could not fetch a valid release JSON from the remote".
    expect(yaml).toMatch(/publish-latest-json/);
    expect(yaml).toMatch(/generate-latest-json\.sh/);
    expect(yaml).toMatch(/latest\.json/);
    const script = readRepo('scripts', 'generate-latest-json.sh');
    expect(script).toMatch(/platforms/);
    expect(script).toMatch(/linux-x86_64/);
    expect(script).toMatch(/windows-x86_64/);
    expect(script).toMatch(/darwin-aarch64/);
  });
});

describe('docs.yml', () => {
  const yaml = readRepo('.github', 'workflows', 'docs.yml');

  it('does not ask GITHUB_TOKEN to create the Pages site', () => {
    // enablement: true fails with "Resource not accessible by integration"
    // when the Pages site was never created by a human/admin token.
    expect(yaml).not.toMatch(/enablement:\s*true/);
    expect(yaml).toMatch(/actions\/configure-pages@v6/);
    expect(yaml).toMatch(/actions\/deploy-pages@v5/);
  });
});

describe('benches/baselines.json + check-perf-regression.sh', () => {
  it('ships a non-empty JSON baseline file with CI-safe merge budget', () => {
    const raw = readRepo('benches', 'baselines.json');
    expect(raw.trim().length).toBeGreaterThan(0);
    const baselines = JSON.parse(raw) as Record<string, unknown>;
    expect(typeof baselines.search_warm_p95_ms).toBe('number');
    expect(typeof baselines.merge_10kb_clean_us).toBe('number');
    // GHA ubuntu-latest measured ~36us; local 18us was too tight.
    expect(baselines.merge_10kb_clean_us as number).toBeGreaterThanOrEqual(35);
    expect(baselines.search_warm_p95_ms as number).toBeGreaterThan(0);
  });

  it('passes the real regression gate on synthetic Criterion output within budget', () => {
    const dir = mkdtempSync(join(tmpdir(), 'onq-perf-'));
    tempDirs.push(dir);
    const benchOut = join(dir, 'bench.txt');
    // Format produced by Criterion (see scripts/check-perf-regression.sh header).
    writeFileSync(
      benchOut,
      [
        'search_warm_p95         time:   [987.56 µs 991.60 µs 999.32 µs]',
        'merge_10kb_clean        time:   [35.226 µs 35.244 µs 35.266 µs]',
        '',
      ].join('\n'),
      'utf8',
    );

    const script = join(repoRoot, 'scripts', 'check-perf-regression.sh');
    const baselines = join(repoRoot, 'benches', 'baselines.json');
    const out = execFileSync(script, [baselines, benchOut], {
      encoding: 'utf8',
      cwd: repoRoot,
    });
    expect(out).toMatch(/search_warm_p95:.*— OK/);
    expect(out).toMatch(/merge_10kb_clean:.*— OK/);
    expect(out).toMatch(/perf gate: OK/);
  });

  it('fails the real regression gate when merge regresses past baseline*1.10', () => {
    const dir = mkdtempSync(join(tmpdir(), 'onq-perf-'));
    tempDirs.push(dir);
    const benchOut = join(dir, 'bench.txt');
    // 100us vs baseline 40 * 1.10 = 44 → must fail.
    writeFileSync(
      benchOut,
      'merge_10kb_clean        time:   [99.0 µs 100.0 µs 101.0 µs]\n',
      'utf8',
    );

    const script = join(repoRoot, 'scripts', 'check-perf-regression.sh');
    const baselines = join(repoRoot, 'benches', 'baselines.json');
    let status = 0;
    let combined = '';
    try {
      combined = execFileSync(script, [baselines, benchOut], {
        encoding: 'utf8',
        cwd: repoRoot,
      });
    } catch (err) {
      const e = err as { status?: number; stderr?: string; stdout?: string };
      status = e.status ?? 0;
      combined = `${e.stdout ?? ''}${e.stderr ?? ''}`;
    }
    expect(status).toBe(1);
    expect(combined).toMatch(/REGRESSION:\s*merge_10kb_clean/);
  });
});
