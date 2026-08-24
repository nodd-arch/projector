# AGNUS PRO (Projector)

A lightweight, keyboard-first scripture projection app for live services — search, browse, or recall a verse instantly and project it to a second window (for a physical projector or screen-sharing in Google Meet/Zoom).

Built with **Tauri v2** (Rust) + **SvelteKit**, backed by a local SQLite database with full-text search across multiple translations.

---

## Features

- Instant search by reference (`Jn 3:16`, `1 Thess 5:16`) or keyword, with book-name aliases and autocomplete
- Book-scoped keyword search (`wisdom understanding; prov`)
- Browse by book (grouped Old/New Testament, filterable) → chapter → verse
- Session history of everything you've projected, reprojectable with one click
- Live preview pane mirrors exactly what's on the projection screen
- Keyboard-driven workflow: switch translations, step through verses, and panic-clear without touching the mouse
- Configurable fonts (with weights), text colors (including separate styling for words of Jesus and bracketed/added text), and background (solid color, gradient, image, or looping video)
- Adjustable output resolution presets for screen-sharing
- Background persists independently of the verse text — clearing the verse never disrupts your background

---

## Requirements

- **Windows 10/11** (current build target)
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Node.js](https://nodejs.org/) and [Bun](https://bun.sh/) (`npm install -g bun` or follow Bun's install docs)
- A `bible.db` SQLite file with the expected schema placed at `src-tauri/bible.db` |  ([Download it here!](https://drive.google.com/file/d/1pWbZSeowKRf1C6LqDP4jO9v7HS5ZbzUo/view?usp=sharing))

---

## Development

Clone the repository, then from the project root:

```powershell
bun install
bun run tauri dev
```

This starts the Vite dev server and launches the app in a native window. Hot-reload works for Svelte/CSS changes; Rust changes and `tauri.conf.json` edits require stopping (`Ctrl+C`) and restarting the command.

---

## Building an installer

```powershell
bun run tauri build
```

Produces a Windows installer at:

```
src-tauri/target/release/bundle/msi/   (.msi)
src-tauri/target/release/bundle/nsis/  (.exe)
```

Either installer is fine to distribute — NSIS (`.exe`) is the more common default for end users.

---

## Keyboard controls

| Key | Action |
|---|---|
| `Ctrl` + `F` | Focus and select the search field |
| `Tab` | Cycle to the next translation |
| `1`–`9` | Jump directly to a translation by position |
| `←` / `→` | Step to the previous/next verse (rolls into the next chapter) |
| `Esc` | Fade the projected text to black; press again to restore |

## Typical workflow

1. Click **Open Projection Window** once at the start of a service. Position it and share that specific window in Meet/Zoom, then leave it alone.
2. Search or browse to a passage, then click **Project** to put a verse on screen.
3. Use `←`/`→` and `Tab`/number keys to move through the passage and switch translations without touching the mouse.
4. Use **Remove Projection** or `Esc` to clear the text between segments — the background stays until you explicitly hide the projection window.
5. Adjust fonts, colors, background, and output resolution any time from **Settings**.

---

## Project structure

```
src/                      SvelteKit frontend
  routes/
    +page.svelte           Control console (search, browse, history, live preview)
    projection/+page.svelte  Output window (what gets projected/shared)
    settings/+page.svelte     Fonts, colors, background, resolution, manual
  lib/
    ScriptureDisplay.svelte  Shared verse-rendering component
    actions/autoFitText.js  Auto-sizing text to fit the canvas
    stores/session.js       Shared session state (survives navigating to Settings)

src-tauri/                Rust backend
  src/
    commands.rs            All Tauri commands (search, navigation, settings, history)
    db.rs / history.rs      SQLite connection management
    models.rs               Shared data structures
    reference.rs            Reference parsing + book alias resolution
  bible.db                 Bundled scripture database (read-only resource)
```

---

## Notes

- Session history clears each time the app is closed and reopened; it is not meant as permanent storage.
- Fonts, colors, background, and output resolution persist across restarts.
- Local background images/videos are referenced by file path; only paths on this machine will resolve correctly if the project is moved to another computer.
