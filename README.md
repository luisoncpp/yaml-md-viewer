# YAML Markdown Viewer

YAML Markdown Viewer is a desktop app for opening, previewing, and exporting `.yaml.md` documents. It compiles one document at a time locally with the pinned [`mdyaml2html`](https://github.com/luisoncpp/html-effectiveness-scripts) library, previews it in a sandboxed child frame, and exports the exact compiled HTML snapshot.

## What is `.yaml.md`?

A `.yaml.md` file is a hybrid Markdown document: ordinary Markdown provides the prose, while fenced YAML blocks describe richer visual components such as notices, cards, data grids, timelines, flowcharts, and code panels. Optional YAML frontmatter controls document-level settings such as the title, layout, and theme.

````markdown
---
title: Project status
layout: wide
---

# This week's update

The release is ready for review.

```yaml
type: notice
variant: success
content: All automated checks passed.
```
````

The format is useful for reports, technical explanations, diagrams, prototypes, and other documents that need more visual structure or interactivity than plain Markdown while remaining readable, token efficient, and friendly to version control. The compiler turns the source into a self-contained HTML file with its styles and component behavior included.

This viewer is for reading and sharing those documents: open a `.yaml.md` source file, inspect the rendered result, present it fullscreen, or save it as standalone HTML.

## Acknowledgements

The visual style is inspired by [Thariq Shihipar (`ThariqS`)](https://github.com/ThariqS) and his [`html-effectiveness`](https://github.com/ThariqS/html-effectiveness) project.

## Development

Requires Node.js, Rust 1.85 or newer, and Windows WebView2 for the currently verified target.

```text
npm install
npm run tauri dev
```

CLI usage: `yamlmdviewer [--] [DOCUMENT.yaml.md]`.

Use the native **File** menu to open a document or save HTML. Use **View → Fullscreen** or `F11` to enter fullscreen; `F11` or `Esc` exits it. Fullscreen hides both the menu bar and the bottom status bar.

The preview uses `iframe.srcdoc` with an opaque `sandbox="allow-scripts"` origin and a preview-only restrictive CSP. Compiler component scripts may run; source-authored scripts are disabled at compilation. The exported HTML is never modified by preview security wrapping.

No installer should be distributed until the upstream compiler project provides or confirms its license.

For an implementation map, security invariants, and future-agent maintenance notes, see [AGENT_GUIDE.md](AGENT_GUIDE.md).
