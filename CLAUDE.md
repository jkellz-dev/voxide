# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run
cargo run

# Tests
cargo test

# Single test
cargo test test_parse_style_foreground

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

Voxide is a Ratatui TUI app following an **Elm-like architecture**:

1. **`tui::Tui`** — owns the terminal and drives two async loops: one polls crossterm events, the other fires `Tick`/`Render` intervals. All events feed into `mpsc::unbounded_channel::<Event>`.

2. **`app::App`** — the main event loop. It receives `Event`s from `Tui`, maps keys to `Action`s via the keybinding table (`config.keybindings[mode]`), then broadcasts actions to all components via a second channel. Components can emit further actions from `update()`.

3. **`action::Action`** — the central message type. Everything from UI lifecycle (`Tick`, `Render`, `Resize`) to domain commands (`Search`, `PlaySelectedStation`, `StationsFound`) flows as an `Action`.

4. **`components::Component` trait** — the unit of UI. Each component implements:
   - `update(&mut self, action: Action) -> Result<Option<Action>>` — state changes
   - `draw(&mut self, f: &mut Frame, area: Rect) -> Result<()>` — rendering
   
   Current components: `Home` (station list + now-playing), `Search` (search popup), `FpsCounter`.

5. **`mode::Mode`** — `Home` or `Search`. Keybindings are scoped per mode. `App.mode` is updated via `Action::Mode(mode)`.

6. **Audio playback** (`models/radio_station.rs::RadioStation::play`) uses three layers:
   - Async tokio task streams HTTP chunks into a shared `Arc<Mutex<VecDeque<u8>>>` (`AudioStream`)
   - `tokio::task::spawn_blocking` runs rodio on a dedicated thread (rodio requires blocking I/O)
   - `broadcast::channel` carries shutdown signals and volume updates between layers

7. **`models::RadioApi`** wraps the `radiobrowser` crate. Searches are dispatched from `Home::search_stations` as a `tokio::spawn` and communicate results back via `Action::StationsFound`.

8. **`config::Config`** — keybindings and styles, loaded from `.config/config.json5` (embedded at compile time as the default) merged with user config files from the platform config dir. Key sequences are parsed from strings like `"<ctrl-a>"` into `Vec<KeyEvent>`.

## Key Files

| File | Role |
|------|------|
| `src/app.rs` | Main event/action loop |
| `src/tui.rs` | Terminal + event stream |
| `src/action.rs` | All action variants |
| `src/components/home.rs` | Primary UI component; owns `StreamState` |
| `src/models/radio_station.rs` | HTTP streaming + rodio playback |
| `src/config.rs` | Keybinding/style parsing |
| `.config/config.json5` | Default keybindings (embedded at compile time) |
