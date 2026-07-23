# Compat vault golden fixture

Encrypted onQ vault snapshot used by `compat_vault` integration tests.

| Field | Value |
|-------|-------|
| Passphrase | `compat-fixture-passphrase` |
| Prompt id | `01COMPATFIXTURE0000000001` |
| Prompt title | `Compat fixture prompt` |
| Theme | `light` |
| Favorite | `true` |
| `schema_version` (at generation) | `3` |
| mongreldb-core (at generation) | `0.64.5` |

## What this catches

Opening this tarball exercises the real MongrelDB on-disk format plus onQ
migrations. Synthetic unit tests that `create_encrypted` with the *current*
engine cannot catch format breaks; this fixture can.

## Regenerate

After an intentional schema or engine change that invalidates old bytes:

```bash
./scripts/generate-compat-vault.sh
```

Commit the updated `compat-vault.tar.gz` and this file together.
