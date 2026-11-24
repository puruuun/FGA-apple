# AGENTS.md

Rules every agent follows in this repository. The design is in [docs/design/0001-rust-core.md](docs/design/0001-rust-core.md).

## Rules

- **Branch and PR.** Work on your own branch in your own worktree and open a PR against `main`. Do not push to `main` and do not merge.
- **Stay in your task's paths.** Never create, edit or delete anything under `fga/`, not even a temporary file: it is a read-only submodule of another project. To run the Kotlin original for expected values, put the throwaway code under `host/`, which depends on it, and remove it before you commit.
- **No device work.** Do not operate a phone, the game, Xcode signing or installation, or repository settings.
- **Never commit** screenshots, screen recordings, exported battle configs (`.fga`), a signing team id, a private network address, or a secret. Screens of the game show account data.
- **No comments.** Write code without comments. The one exception is a single line for a fact the code cannot show, such as a measured value or an external constraint, with where it comes from. Never write what the code does, and never write in a test why it passes.
- **A comment is not evidence.** When you test or verify, decide from what the code does when it runs. A comment, a name or a commit message saying something works does not make it pass.
- **A test must be able to fail.** Each new test fails when the code it guards is disabled: check that, then restore the code. Do not change an expected value, a threshold or a test to get a pass. Report numbers as they are.
- **A cached result is not a run.** When you report that tests pass, they ran in this session: force them to run (`gradlew :host:test --rerun`, `cargo test` after a real change) and report the counts.
- **Fakes stay honest.** A fake or a mock returns only what the real thing can return in that situation.
- **All servers.** Do not add logic that works for one game server only. Template images fall back to `En` when a server has none.
- **When blocked, stop and report.** A denied tool, a missing tool, a failing requirement: report the exact message. Do not work around it and do not install software.
- **Run the checks for what you touched** before you push, and report the results with what you did not verify.
- **Language.** Reports to the owner are in Korean. Commits and PRs are in English.

## Checks

```bash
./gradlew :host:test --rerun                  # host/  (needs the fga submodule: git submodule update --init)
pio run -d firmware                           # firmware/
python -m compileall -q tools mac-capture     # Python tools (on the Windows machine the interpreter is `py`, not `python`)
python -m unittest discover tools/ci/tests    # tools/ci/
python tools/ci/hygiene.py                    # nothing that must not be committed
```

```bash
cd core
cargo fmt --all --check                       # core/
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --target aarch64-apple-ios -p vision -p automata -p scripts
cargo deny check
```
