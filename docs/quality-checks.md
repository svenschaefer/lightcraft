# Quality checks

Native workspace tests compile the browser package as an `rlib`. `cargo xtask web`
explicitly requests its WebAssembly `cdylib`, avoiding the Windows DLL export limit
without omitting browser-package tests.

The general CI workflow runs cargo xtask ci on every pull request and push to main, on
Linux and Windows. It checks formatting, Clippy, optional HEIF support, workspace tests,
parity references, dependency layering, assets and WebAssembly compilation. Linux uses
pinned first-party craft-fonts; Windows checks the optional-font fallback build.

## Catalog hierarchy regressions

```sh
cargo test -p lightcraft-catalog --lib
cargo test -p lightcraft-catalog --lib tests_album_validation
```

Snapshots and journal snapshots must contain an acyclic album hierarchy. Every parent
must exist and be a folder. Validate the hierarchy before replaying the journal or
repairing a torn tail, so a corrupt snapshot is refused without rewriting the stored
snapshot or log.

Moving an album also tracks visited parents. A pre-existing cycle must produce an error
before the move changes the catalog. The regressions cover corrupt snapshots, unchanged
journal storage after refusal, missing parents and a deep valid hierarchy.

The aggregate CI check is **quality gate**. Select it in branch rules after its first
GitHub run; adding the workflow does not make it a required merge check automatically.
