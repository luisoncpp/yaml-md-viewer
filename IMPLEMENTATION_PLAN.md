# yamlmdviewer implementation handoff

## 1. Objective

Build a Tauri 2 desktop application that opens a single `*.yaml.md` document, compiles it with the Rust library from `luisoncpp/html-effectiveness-scripts`, previews the resulting self-contained HTML, and exports that exact HTML through a native Save As dialog.

Required user-visible features:

- Open one `*.yaml.md` file from a native file picker.
- Open one `*.yaml.md` file supplied as a positional command-line argument.
- Forward a file from a second invocation to the already-running application.
- Preview the compiled HTML in the application window.
- Save the last successful compilation as a standalone `.html` file.
- Toggle fullscreen from the native menu bar or `F11`; leave fullscreen with `Esc`.
- Show actionable errors without destroying the last successful preview.

The first verified target is Windows/WebView2. Keep the design portable to Linux and macOS, but do not claim those platforms are verified without testing them.

## 2. Fixed product decisions

- Tauri 2, Rust backend, Vite plus vanilla TypeScript frontend.
- One native window and one document at a time.
- The trusted application shell remains loaded at all times. Never replace the main webview document with generated HTML.
- All filesystem reads and writes happen in Rust. Do not add the Tauri filesystem plugin.
- Native dialogs select paths only. Rust validates and uses the selected path.
- Compile with source-authored custom scripts disabled.
- Preserve compiler-owned scripts required by interactive components.
- Keep the last successful compiled HTML in Rust application state. Save As writes this exact snapshot; it does not reread or recompile the source.
- Use a sandboxed child preview. Do not inject compiled HTML into the shell DOM.
- Pin the compiler dependency to commit `6705b94d7b74d4279841c75c3eb03b0df2df8b1d`, the commit that separates the library and CLI packages.
- Use a Rust toolchain compatible with edition 2024 (Rust 1.85 or newer).
- Commit both Rust and JavaScript lockfiles.

## 3. Upstream compiler contract

Cargo dependency:

```toml
mdyaml2html = {
  git = "https://github.com/luisoncpp/html-effectiveness-scripts",
  rev = "6705b94d7b74d4279841c75c3eb03b0df2df8b1d"
}
```

Compilation must use this shape:

```rust
use mdyaml2html::{CompileOptions, HtmlOptions, compile};

let options = CompileOptions {
    html: Some(HtmlOptions {
        enable_custom_scripts: Some(false),
        ..Default::default()
    }),
};

let compiled = compile(&source, &options)?;
let html = compiled.html;
```

Important upstream details:

- `compile` consumes UTF-8 source text and returns a self-contained HTML string.
- Generated CSS and component JavaScript are inlined.
- `enable_custom_scripts: Some(false)` strips source-authored `<script>` elements while retaining compiler asset scripts.
- The current upstream metadata field is misspelled `metadatata`. Do not expose that typo to the frontend. Map the title into this application's own model.
- Script stripping alone is not a complete security boundary: raw HTML can still include links, event attributes, remote resources, forms, or embedded content. The preview sandbox and CSP remain mandatory.
- The upstream repository currently has no `LICENSE` file and its manifest does not declare a license. Treat installer distribution as blocked until the owner explicitly adds or confirms a license. Local development is not blocked.

## 4. Proposed repository layout

The exact generated boilerplate may vary slightly, but preserve these responsibilities:

```text
/
  IMPLEMENTATION_PLAN.md
  README.md
  package.json
  package-lock.json
  tsconfig.json
  vite.config.ts
  index.html
  src/
    main.ts                 # DOM wiring and orchestration only
    styles.css
    models.ts               # frontend mirrors of serializable Rust contracts
    api.ts                  # typed invoke/listen wrappers
    preview.ts              # iframe creation, revision checks, sandbox setup
    fullscreen.ts           # frontend fullscreen-state synchronization
  src-tauri/
    Cargo.toml
    Cargo.lock
    rust-toolchain.toml
    tauri.conf.json
    capabilities/
      default.json
    src/
      main.rs               # thin binary entry point
      lib.rs                # builder, plugins, setup, managed state
      error.rs              # serializable AppError and error codes
      models.rs             # DocumentSnapshot and frontend DTOs
      document.rs           # pure DocumentService
      arguments.rs          # pure argv parsing and path resolution
      commands.rs           # narrow Tauri command adapters
      instance.rs           # startup and second-instance orchestration
  tests/fixtures/
    valid.yaml.md
    interactive.yaml.md
    malformed.yaml.md
    custom-script.yaml.md
```

Keep Tauri-specific types out of `document.rs` and `arguments.rs` so their behavior is covered by ordinary Rust unit tests.

## 5. Rust domain model and state

Use application state protected by a mutex. Never hold its lock while reading, compiling, writing, emitting events, or interacting with a window.

```rust
#[derive(Default)]
pub struct AppState {
    pub next_revision: u64,
    pub current: Option<DocumentSnapshot>,
}

#[derive(Clone)]
pub struct DocumentSnapshot {
    pub revision: u64,
    pub source_path: PathBuf,
    pub display_title: String,
    pub compiled_html: String,
}
```

Frontend DTO returned by commands and events:

```rust
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub revision: u64,
    pub source_path: String,
    pub display_title: String,
    pub compiled_html: String,
}
```

Rules:

1. Read and compile outside the state lock.
2. Only after compilation succeeds, acquire the lock, allocate a strictly increasing revision, and replace `current`.
3. A failed open never clears or modifies `current`.
4. Frontend ignores any event/response whose revision is older than the revision already displayed.
5. Export clones the current HTML while holding the lock briefly, releases the lock, then writes it.

Define stable error codes such as:

- `invalid_extension`
- `file_not_found`
- `not_a_file`
- `read_failed`
- `invalid_utf8`
- `compile_failed`
- `no_document`
- `write_failed`
- `invalid_arguments`

Send a safe user-facing message plus the stable code. Preserve detailed source chains in Rust logs; do not leak arbitrary internals into the UI.

## 6. DocumentService behavior

Implement pure/testable functions for:

### Validate input

- Require an existing regular file.
- Validate by filename suffix, not `Path::extension()`, because the final extension is only `md`.
- Accept a case-insensitive `.yaml.md` suffix on Windows. For predictable cross-platform behavior, accepting it case-insensitively everywhere is reasonable.
- Reject directories and other file types with `invalid_extension` or `not_a_file` as appropriate.

### Compile input

- Read bytes and explicitly decode UTF-8 so invalid encoding maps to `invalid_utf8`.
- Call `mdyaml2html::compile` with custom source scripts disabled.
- Derive `display_title` from compiler metadata when present and nonblank; otherwise use the filename without `.yaml.md`.
- Retain the returned HTML byte-for-byte in the snapshot.

### Export snapshot

- The requested output path comes from a native save dialog.
- If it does not end in `.html` case-insensitively, append `.html`.
- Write the exact `compiled_html` string from the last successful snapshot.
- Return the final written path.
- Native dialog behavior owns overwrite confirmation.

## 7. Tauri command contract

Expose only the commands needed by the shell:

```text
open_document(path: string) -> DocumentView
get_current_document() -> DocumentView | null
save_current_document(path: string) -> SaveResult
```

`SaveResult` contains the final path after extension normalization.

Fullscreen can use the scoped Tauri window JavaScript API directly if the capability grants only the necessary methods. If keeping all privileged operations in Rust produces a materially smaller capability, expose:

```text
set_fullscreen(fullscreen: bool) -> bool
get_fullscreen() -> bool
```

Choose one approach and test it; do not implement duplicate pathways.

The dialog plugin is used by TypeScript for Open and Save dialogs. It returns selected paths, which are then passed to the Rust commands. Configure filters as:

```text
Open: name "YAML Markdown", extensions ["yaml.md"]
Save: name "HTML", extensions ["html"]
```

If a platform's native dialog does not handle the compound `yaml.md` filter correctly, use `md` in the dialog for discoverability but keep strict `.yaml.md` validation in Rust. Verify this on Windows rather than assuming it works.

## 8. Startup and single-instance command-line flow

Do not add a general-purpose CLI parsing plugin for one positional path. Keep parsing in `arguments.rs`.

Supported syntax:

```text
yamlmdviewer [--] [DOCUMENT.yaml.md]
```

Parsing rules:

- Ignore argv element zero.
- `--` ends option parsing.
- Before `--`, reject unknown option-looking values rather than treating them as paths.
- Select at most one positional file. More than one produces `invalid_arguments`.
- Preserve OS strings until path construction so Unicode paths work.
- Resolve a relative path against the invoking process/current working directory.
- Paths with spaces must work without special application-level handling; the shell supplies a single argv item when quoted correctly.

### Cold start

1. Register `tauri-plugin-single-instance` before every other plugin.
2. During setup, parse `std::env::args_os()` with the current process directory.
3. Create/show the main window and let the shell register its event listeners.
4. Load the startup path without assuming the frontend listener already exists.
5. Store success in `AppState`; emit an event when possible.
6. On frontend initialization, call `get_current_document()` so an early event can never be lost.

### Later invocation

The single-instance callback receives argv and the invoking working directory:

1. Parse and resolve the incoming path against that callback's working directory.
2. Read and compile it outside the UI thread where needed.
3. On success, replace state and emit `document-opened` with `DocumentView`.
4. On failure, leave state unchanged and emit `document-error` with a typed error payload.
5. Restore/unminimize, show, and focus the main window regardless, so the user sees the result.

If the plugin callback cannot perform asynchronous compilation safely, send the request to a worker/channel owned by application state and perform the work there. Do not block window message processing with a large compile.

## 9. Frontend state machine

Use a small explicit state model rather than scattered DOM flags:

```text
empty
loading(previousDocument?)
ready(document)
error(error, previousDocument?)
```

Behavior:

- Open dialog cancellation is a no-op.
- During a new open, retain the existing preview and show a non-destructive loading indicator.
- On success, replace the preview and update title/path/save state.
- On failure, keep the prior preview, show the error, and leave Save enabled only if a successful snapshot still exists.
- Save dialog defaults to `<source basename>.html` next to the source when the dialog API supports it.
- Save cancellation is a no-op.
- After saving, show a short nonmodal success status with the final path.
- Subscribe to `document-opened` and `document-error` events for second-instance requests.
- Call `get_current_document()` after listeners are registered and reconcile by revision.

Minimum accessible UI:

- Native File menu with Open and Save as HTML; Save is disabled with no successful document.
- Native View menu with a Fullscreen command; `F11` is also available from either shell or preview focus.
- Visible document title and optionally abbreviated path with the full path available as a tooltip.
- Bottom status/error bar using appropriate `aria-live` behavior.
- Preview fills remaining window space.

## 10. Preview security and WebView2 spike

This is an early implementation gate, not late hardening.

Preferred design:

```html
<iframe sandbox="allow-scripts"></iframe>
```

Do not add `allow-same-origin`, `allow-top-navigation`, `allow-forms`, `allow-popups`, or `allow-downloads` without a demonstrated compiler requirement and a security review.

The generated document needs inline CSS and compiler-owned inline JavaScript. The shell should have a restrictive CSP. Before building the polished shell, create a spike that proves the chosen delivery method on Windows WebView2.

Spike acceptance checks:

1. Static prose and styled YAML components render.
2. At least one built-in interactive component works.
3. A source-authored `<script>` is absent and does not run.
4. The child cannot read or modify `window.top.document`.
5. The child cannot access Tauri globals or invoke IPC.
6. Remote images, fetch/XHR/WebSocket, objects, frames, forms, popups, downloads, and top navigation are blocked.
7. Clicking a link cannot replace the application shell.
8. The shell CSP does not have to be weakened globally to make the preview work.

Start by assigning generated HTML through `iframe.srcdoc`. Verify whether WebView2 applies/inherits CSP in a way that blocks required inline component scripts. If it does, do not add broad `unsafe-inline` privileges to the shell. Instead, implement a narrowly scoped local preview delivery mechanism with a separate preview CSP and opaque/sandboxed origin. Document the final mechanism and why it passed the checks above.

The HTML saved to disk must remain the original compiler output. Never export the preview wrapper or any preview-only CSP injection.

## 11. Fullscreen behavior

- The native View menu command and `F11` toggle the native window fullscreen state.
- Shell key handling plus a narrow preview-to-shell `postMessage` bridge handle `F11` and `Esc` even when focus is inside the sandboxed preview.
- `Esc` exits fullscreen but does nothing when not fullscreen.
- Query the actual native fullscreen state after each transition; do not assume the requested state succeeded.
- Listen for window resize/state changes as needed so shell visibility remains synchronized if fullscreen changes outside the menu.
- Fullscreen applies to the entire application window and hides both the native menu bar and bottom status bar.

## 12. Capability and configuration policy

- Register the single-instance plugin first.
- Register the dialog plugin.
- Do not include filesystem, shell, HTTP, opener, or broad process permissions.
- Grant only the dialog open/save permissions, event/listen permissions required for document forwarding, and the exact window fullscreen permissions selected in section 7.
- Keep Tauri global APIs disabled unless imports cannot serve the need.
- Use the restrictive default shell CSP and a separately reasoned preview policy.
- Ensure navigation to arbitrary external origins is not allowed by Tauri configuration.
- Add logging suitable for diagnosing Rust errors without recording document content.

## 13. Detailed implementation order

Terra should execute these as separate, verifiable milestones and stop on a failed gate rather than building UI on an unproven foundation.

### Milestone 1: Scaffold and lock dependencies

- Scaffold Tauri 2 plus vanilla TypeScript/Vite.
- Add pinned compiler dependency, dialog plugin, and single-instance plugin.
- Add `rust-toolchain.toml` with Rust 1.85 or newer.
- Generate and commit lockfiles.
- Confirm an empty app builds and launches on Windows.

Exit gate: frontend build, `cargo check`, and Tauri development launch succeed.

### Milestone 2: Pure Rust compiler service

- Implement errors, input validation, UTF-8 read, compiler invocation, title derivation, and exact export.
- Add fixtures and unit tests before wiring Tauri.

Exit gate: all document-service tests pass, including custom script removal and preservation of compiler component scripts.

### Milestone 3: State and arguments

- Implement snapshot/revision state without holding locks across I/O.
- Implement argv parsing and relative path resolution.
- Test absolute, relative, Unicode, spaced, `--`, missing, and multiple positional arguments.

Exit gate: deterministic pure Rust tests pass.

### Milestone 4: Minimal Tauri integration

- Register plugins in the required order.
- Add minimal commands and typed events.
- Wire cold-start and second-instance loading.
- Restore/show/focus the existing window on later invocations.

Exit gate: a temporary minimal shell can open a file through IPC, cold start, and a second invocation without losing state/events.

### Milestone 5: Preview security spike

- Implement the sandboxed preview delivery experiment.
- Run every check in section 10 on WebView2.
- Record the final selected delivery mechanism in this document or an architecture note.

Exit gate: built-in interactivity works and the child has no parent/IPC/network/navigation escape. Do not proceed if both conditions cannot be satisfied.

### Milestone 6: Product UI

- Implement the native menu, state machine, dialogs, preview, bottom status bar, and title/path presentation.
- Reconcile startup state and events by revision.
- Ensure failed opens preserve the prior preview.

Exit gate: open/save/error/cancellation flows pass manual and automated frontend checks.

### Milestone 7: Fullscreen and polish

- Add shell and sandbox-preview handling for `F11` and `Esc`, native-state reconciliation, and fullscreen shell visibility.
- Test while empty, ready, loading, and showing errors.

Exit gate: every fullscreen acceptance case passes.

### Milestone 8: Release verification and documentation

- Run formatting, linting, Rust tests, frontend tests/build, and Tauri debug/release builds.
- Perform the Windows manual test matrix.
- Document prerequisites, commands, CLI syntax, limitations, security policy, and verified platforms.
- Resolve upstream licensing before producing or distributing an installer.

Exit gate: all automated checks pass; manual results and any platform limitations are documented.

## 14. Acceptance test matrix

### Opening and compilation

- Open-dialog cancellation changes nothing.
- Valid lowercase `.yaml.md` opens.
- Mixed-case `.YAML.MD` opens.
- A path with spaces and Unicode opens.
- A missing file returns `file_not_found`.
- A directory returns `not_a_file`.
- A plain `.md` file returns `invalid_extension`.
- Invalid UTF-8 returns `invalid_utf8`.
- Malformed YAML returns `compile_failed` with a useful message.
- Every failure leaves the previous successful document and Save capability intact.

### Preview and security

- Prose, styles, and every representative component render.
- Built-in interactive component behavior works.
- Source-authored script does not appear or execute.
- Preview cannot access parent DOM or Tauri IPC.
- Preview cannot fetch remote content or navigate the top-level window.
- Preview links/forms/popups/downloads cannot escape the sandbox.

### Export

- Save is disabled before the first successful open.
- Save-dialog cancellation writes nothing and changes no state.
- Default name converts `report.yaml.md` to `report.html`.
- Missing `.html` suffix is appended once.
- Existing `.HTML` suffix is accepted.
- Exported bytes equal the last successful compiler HTML snapshot.
- If the source changes on disk after opening, export still writes the displayed snapshot.
- Write failure is reported without losing the current document.

### Command line and single instance

- Launch with no path shows the empty state.
- Launch with absolute path opens the document.
- Launch with relative path resolves against the invoking working directory.
- Quoted spaces, Unicode, and `--` work.
- Unknown options and multiple positional paths produce a typed error.
- A second invocation reuses the current process, opens the new document, and focuses/restores the window.
- A failed second-instance open keeps the existing preview and displays the error.
- An early startup load is recovered by `get_current_document()` even if its event fired before frontend listeners existed.

### Fullscreen

- Native View menu command enters and exits fullscreen.
- `F11` enters and exits fullscreen.
- `Esc` exits fullscreen.
- `Esc` outside fullscreen is a no-op.
- Native menu and bottom status bar are hidden in fullscreen and restored on exit.

### Quality gates

Run the repository-equivalent commands for:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm test
npm run build
npm run tauri build
```

If no frontend test runner is initially scaffolded, add a lightweight one only for logic with real regression value: state transitions, revision reconciliation, filename derivation, and keyboard behavior.

## 15. Explicit non-goals for the first implementation

- Editing `.yaml.md` source.
- Watching files and automatic reload.
- Multiple tabs, multiple documents, or multiple windows.
- Recent-files history.
- File association registration or double-click OS integration.
- Drag and drop.
- Printing or PDF export.
- External URL opening.
- Installer distribution before upstream licensing is clarified.

Do not add these opportunistically. Keep the first version narrow and make every required path reliable.
