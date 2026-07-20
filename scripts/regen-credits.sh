#!/usr/bin/env bash
# Regenerate legal inventories under crates/onq-app/legal/ after dependency changes.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 - <<'PY'
import json, pathlib, subprocess
from collections import defaultdict

ROOT = pathlib.Path(".").resolve()
LEGAL = ROOT / "crates/onq-app/legal"
LEGAL.mkdir(parents=True, exist_ok=True)

meta = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--manifest-path", "Cargo.toml"],
        text=True,
    )
)
pkgs_by_id = {p["id"]: p for p in meta["packages"]}
used, seen = [], set()
for n in meta["resolve"]["nodes"]:
    p = pkgs_by_id.get(n["id"])
    if not p or p["name"] in ("onQ", "onq-core", "onq-plugin-sdk", "onq-test-utils"):
        continue
    key = (p["name"], p["version"])
    if key in seen:
        continue
    seen.add(key)
    used.append(
        {
            "name": p["name"],
            "version": p["version"],
            "license": p.get("license") or "UNKNOWN",
            "repository": p.get("repository") or "",
        }
    )
used.sort(key=lambda x: x["name"].lower())
(LEGAL / "crates.json").write_text(json.dumps(used, indent=2) + "\n")

pkg_json = json.loads((ROOT / "package.json").read_text())
all_deps = {**pkg_json.get("dependencies", {}), **pkg_json.get("devDependencies", {})}
packages = []
for name in sorted(all_deps):
    meta_path = ROOT / "node_modules" / name / "package.json"
    role = "runtime" if name in pkg_json.get("dependencies", {}) else "dev"
    if not meta_path.exists():
        packages.append(
            {
                "name": name,
                "version": str(all_deps[name]).lstrip("^~"),
                "license": "UNKNOWN",
                "repository": "",
                "role": role,
            }
        )
        continue
    m = json.loads(meta_path.read_text())
    lic = m.get("license", "UNKNOWN")
    if isinstance(lic, dict):
        lic = lic.get("type", "UNKNOWN")
    repo = m.get("repository")
    if isinstance(repo, dict):
        repo = repo.get("url", "")
    if isinstance(repo, str):
        repo = repo.replace("git+", "").replace(".git", "")
    packages.append(
        {
            "name": name,
            "version": m.get("version", ""),
            "license": lic if isinstance(lic, str) else str(lic),
            "repository": repo or "",
            "role": role,
        }
    )
(LEGAL / "npm-packages.json").write_text(json.dumps(packages, indent=2) + "\n")

by_lic = defaultdict(list)
for c in used:
    by_lic[c["license"]].append(c)
lines = [
    "# Third-Party Licenses (Rust crates)",
    "",
    "Rust crates included in this build of onQ, grouped by license expression.",
    "onQ is licensed under **GPL-3.0-only**.",
    "",
    "Regenerate with `scripts/regen-credits.sh`.",
    "",
    "## Licenses in use",
    "",
]
for lic, crates in sorted(by_lic.items(), key=lambda x: (-len(x[1]), x[0])):
    lines.append(f"- **{lic}** ({len(crates)} crates)")
lines.append("")
for lic, crates in sorted(by_lic.items(), key=lambda x: x[0].lower()):
    lines.append(f"## {lic}")
    lines.append("")
    for c in sorted(crates, key=lambda x: x["name"].lower()):
        extra = f" — {c['repository']}" if c["repository"] else ""
        lines.append(f"- `{c['name']}` {c['version']}{extra}")
    lines.append("")
(LEGAL / "third-party.md").write_text("\n".join(lines) + "\n")

by_npm = defaultdict(list)
for p in packages:
    by_npm[p["license"]].append(p)
nlines = [
    "# Frontend (npm) Third-Party Licenses",
    "",
    "Packages from package.json. onQ is **GPL-3.0-only**.",
    "",
    "## Packages in use",
    "",
]
for lic, pkgs in sorted(by_npm.items(), key=lambda x: (-len(x[1]), x[0])):
    nlines.append(f"- **{lic}** ({len(pkgs)} packages)")
nlines.append("")
for lic, pkgs in sorted(by_npm.items(), key=lambda x: x[0].lower()):
    nlines.append(f"## {lic}")
    nlines.append("")
    for p in sorted(pkgs, key=lambda x: x["name"].lower()):
        nlines.append(f"- `{p['name']}` {p['version']} ({p['role']})")
    nlines.append("")
(LEGAL / "npm-third-party.md").write_text("\n".join(nlines) + "\n")
print(f"Wrote {len(used)} crates, {len(packages)} npm packages under {LEGAL}")
PY
