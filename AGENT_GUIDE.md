# Future-agent implementation guide

This is the map of the current implementation. Read this before changing behavior; `IMPLEMENTATION_PLAN.md` remains the product specification and this guide describes where that specification lives in code.

## Fast orientation

```text
Native OS / CLI / state
  src-tauri/src/lib.rs ── builder and startup
  src-tauri/src/instance.rs ── later invocations
  src-tauri/src/commands.rs ── IPC adapters
  src-tauri/src/document.rs ── compiler and disk I/O
          │ document-opened / document-error
          ▼
Frontend shell
  src/main.ts ── UI state and orchestration
  src/api.ts ── typed Tauri APIs
  src/preview.ts ── sandboxed preview delivery
  src/fullscreen.ts ── keyboard/native fullscreen synchronization
```

## Feature-to-file map

| Feature or decision | Primary implementation | Notes |
| --- | --- | --- |
| Tauri application, plugin order, managed state, startup path | `src-tauri/src/lib.rs` | Single-instance is deliberately registered before dialog. Startup compiles before the shell may be listening; the frontend recovers via `get_current_document`. |
| One-document snapshot and strictly increasing revisions | `src-tauri/src/models.rs`, `src-tauri/src/commands.rs` | `AppState.current` holds the last successful snapshot. Compile happens before locking; save clones HTML while locked, then writes after releasing it. |
| `.yaml.md` validation, UTF-8 decoding, compiler call, title fallback, export extension normalization | `src-tauri/src/document.rs` | Never use `Path::extension()` for input validation. Source scripts are disabled with `enable_custom_scripts: Some(false)`. Keep `compiled_html` byte-for-byte for export. |
| Error contract | `src-tauri/src/error.rs` | UI-safe `code` and `message`; detailed errors are logged with `eprintln!`. Keep error codes stable. |
| CLI parsing and relative path resolution | `src-tauri/src/arguments.rs` | Supports `yamlmdviewer [--] [DOCUMENT.yaml.md]`; preserves `OsString` through parsing and rejects unknown options/multiple paths. |
| Open/current/save IPC | `src-tauri/src/commands.rs`, `src/api.ts` | Commands are `open_document`, `get_current_document`, `save_current_document`. Fullscreen has separate Rust commands. |
| Second invocation | `src-tauri/src/instance.rs` | The callback immediately restores/focuses the window and spawns compilation off the callback thread. It emits `document-opened` on success or `document-error` on failure. |
| Shell UI and non-destructive errors | `src/main.ts`, `src/styles.css` | Native menu actions are delivered as shell events. `current` is only replaced on a successful response; a failed open therefore preserves preview and Save availability. Revision reconciliation ignores stale documents. |
| Native Open/Save dialogs | `src/main.ts` | Dialogs return only paths. Rust performs all document reads and writes. Dialog cancellation is a no-op. |
| Sandboxed preview and system color mode | `src-tauri/src/preview.rs`, `src/preview.ts`, `src/styles.css` | A narrow local `yamlmdpreview` protocol serves generated HTML to `iframe.src`, with `sandbox="allow-scripts"`. The preview URL carries the system color mode so Rust can apply dark mode to the document root before its assets load. A response CSP blocks network, objects, frames, media, and forms while allowing compiler-owned inline styling/scripts. Never export the injected preview wrapper or color-mode attribute. |
| Shell CSP and narrow permissions | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json` | Do not add filesystem, shell, HTTP, opener, or broad process permissions without a new security review. |
| Native menu and fullscreen control | `src-tauri/src/menu.rs`, `src-tauri/src/commands.rs`, `src/fullscreen.ts`, `src/preview.ts` | The shell handles `F11` and `Esc` directly, while a narrow `postMessage` bridge handles them when the sandboxed preview has focus. Fullscreen hides the native menu and the frontend status bar; the frontend reconciles native state after transitions and on resize. |
| Frontend API contracts | `src/models.ts`, `src/api.ts` | Rust serializes camelCase DTOs, mirrored here. |
| Filename default for Save dialog | `src/filenames.ts` | Test coverage is in `src/filenames.test.ts`. |
| Dependencies and toolchain | `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/rust-toolchain.toml` | The compiler Git revision is intentionally pinned. The project uses the installed MSVC stable toolchain because the local GNU toolchain lacks `dlltool`. |
| Icons | `src-tauri/icons/app.svg` and generated files | `icon.ico` is needed by the Windows Tauri build. Regenerate with `npm run tauri icon .\\src-tauri\\icons\\app.svg` after altering the SVG. |

## Invariants that changes must preserve

- The trusted shell must stay loaded; do not navigate the app webview to compiled HTML or inject compiled HTML into its DOM.
- The preview must remain sandboxed without `allow-same-origin`, top navigation, popups, forms, downloads, or child frames.
- All document file I/O stays in Rust. The dialog plugin only chooses paths.
- Opening failures never clear `AppState.current`; export always writes the last successful, already compiled snapshot.
- The compiler invocation must keep source-authored custom scripts disabled while retaining compiler component scripts.
- Do not hold the `AppState` mutex during filesystem work, compilation, event emission, or window operations.
- Frontend consumers must reconcile by revision, because events and command responses can arrive out of order.
- Keep one window and one document; tabs, watching, editing, external navigation, and drag/drop are explicit non-goals.

## Tests and fixtures

- Rust unit tests are colocated in `src-tauri/src/arguments.rs` and `src-tauri/src/document.rs`.
- Frontend filename behavior is tested in `src/filenames.test.ts`.
- Representative input fixtures are in `tests/fixtures/`. `valid.yaml.md` and `custom-script.yaml.md` are currently used by Rust tests; `interactive.yaml.md` and `malformed.yaml.md` are reserved for manual/security coverage.

Run the checks from the repository root unless noted:

```text
npm test
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Verification boundaries

Automated frontend and Rust checks pass. The Windows release build was started but exceeded the execution environment's two-minute command limit; rerun `npm run tauri build` on a development machine. The WebView2 preview design is implemented, but its full manual security matrix in `IMPLEMENTATION_PLAN.md` (interactive component behavior, network/navigation escape attempts, and parent/IPC isolation) still needs to be recorded on a Windows/WebView2 machine before claiming it as verified.

## Documentation ownership

- Update this file when moving a feature or changing an invariant.
- Update `README.md` for user-facing behavior and prerequisites.
- Update `IMPLEMENTATION_PLAN.md` only to record verified architecture/security decisions or changed product requirements; do not silently weaken its acceptance criteria.
