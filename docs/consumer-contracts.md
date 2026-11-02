# Native consumer integration contracts

What an app (rdpapp's `rdpapp-fluent`, rustTorrent's `rtbit-fluent`, email-rs's
`email-desktop-fluent`) can rely on when it uses `VideoSurface`, `TerminalView` /
`TerminalGrid` and the `fluent-app` pickers. The source of truth is the rustdoc in
`crates/fluent-primitives/src/{video_surface,terminal_view,terminal_grid}.rs` and
`crates/fluent-app/src/pickers.rs`; this page states the rules a consumer must follow.
Windows 10/11 x64 and Linux x64 (X11, Wayland) only.

## 1. Video frames (`VideoSurface`)

- **Frame format.** Tightly packed RGBA8, rows top to bottom, no row padding, each side
  `1..=MAX_VIDEO_FRAME_DIMENSION` (8192). `submit_frame(width, height, pixels, cx)` takes
  a whole frame; `submit_region(rect, pixels, cx)` patches a rectangle of the last frame.
- **Ownership.** The caller owns the frame source and its pixel buffers. The surface keeps
  its own CPU copy of the last frame (for region patches) and owns the GPU texture: it
  allocates on the first frame, reallocates when the size changes, and frees it when the
  view is released. Do not hold or free texture ids.
- **Cadence.** Uploads happen on the next paint, not on submit. Submitting several frames
  between paints uploads only the last whole frame (dirty regions accumulate), so a source
  may submit at its own rate; pacing to the display refresh only saves copying. Submit from
  the UI thread (inside `Entity::update`); a decoder thread hands frames over to it.
- **First frame and size changes.** `submit_region` before any frame fails with
  `RegionOutOfBounds`. Submit a whole frame whenever the source size changes (rdpapp
  compares the shown `frame_size()` to the framebuffer and falls back to `Whole`).
- **Resize feedback.** `VideoSurfaceEvent::Resized` carries the new size in device
  pixels. A remote source should be asked for that size; send it only when it differs
  from the last one sent, and only once the stream is open.
- **Input mapping.** After each paint `layout()` gives the frame rectangle; use
  `VideoLayout::to_frame` (or `to_frame_clamped` for drags) to map window positions to
  frame pixels. It is `None` before the first frame.
- **Errors.** `VideoFrameError` (`BadSize`, `BadLength`, `RegionOutOfBounds`) is returned
  from submit, and the previous frame stays on screen. The surface never panics on
  bad input. Consumers choose whether to log or drop; rdpapp drops, because its bridge
  already reports undecodable frames.
- **Tests.** gpui's test platform does not render video; use `VideoSurface::placeholder`
  and a `VideoRecorder` to assert allocations, uploads and paints.
- **Filtering.** `Fit` is linearly filtered; `Native` is one frame pixel per device pixel.
  Nearest-neighbour scaling is not available.

## 2. Terminal (`TerminalGrid` and `TerminalView`)

- **Split of work.** The view does no VT parsing and owns no pty. The caller owns an
  `Entity<TerminalGrid>`, writes cells from its emulator, and calls `cx.notify()` on the
  grid; the view observes it and repaints. Rows are cached by line id and change counter,
  so only changed rows are reshaped. Mutate through the grid API so versions are bumped.
- **Cell sizing.** The view measures one monospace cell (`cell_metrics()`, `None` until
  the first paint) and fits as many whole cells as its bounds hold, at least 1x1
  (`CellMetrics::grid_size`). When the count changes it calls `TerminalGrid::resize` and
  emits `TerminalViewEvent::Resize { cols, rows }`. The consumer forwards that to its pty
  or remote side; it must not resize the grid to a different size itself, or the view
  will resize it back on the next paint. Give the view a definite size from its parent
  (flex-grow or fixed); an unsized view fits to 1x1.
- **Resize behaviour.** Growing pulls lines out of scrollback; shrinking drops blank lines
  below the cursor, then pushes top lines into scrollback. Sizes below 1 are raised to 1.
  Font family and size (`font_family`, `font_size`) must be set before the first paint;
  changing them later changes the cell size and so the grid size.
- **Fonts.** The default is the system `Consolas` (Windows) or `DejaVu Sans Mono`
  (elsewhere); no font is bundled. Glyphs the font lacks come from a fallback font and may
  not align to the cell grid.
- **Focus and input.** The view's focus handle is its inner `InputCapture`'s (`Focusable`).
  Click, or focus and press Enter, to grab input; Escape or Control+Alt releases (and blur
  releases). While grabbed every key press/release is emitted as
  `TerminalViewEvent::Key` (logical `key`, W3C `code`, physical scancode via the
  capture, `pressed`, `is_held`, `modifiers`) except Control+Shift+C, which copies. Map
  keys to bytes in the consumer (rdpapp acts on `pressed: true` and ignores releases).
  `GrabChanged` reports grab state.
- **Selection and clipboard.** Drag selects, Shift+click extends, wheel scrolls the
  scrollback viewport (`scroll_offset`). A key press clears the selection and returns to
  the live screen. Copy writes to the system clipboard and then emits `Copied(text)`;
  a consumer that gates clipboard access (rdpapp's trust prompt) must clear the
  selection while denied, and may overwrite the clipboard from `Copied` if a copy raced
  the decision.
- **Accessibility.** `Terminal` role, `label(..)`, visible text as value.

## 3. Pickers (`fluent_app::pickers`)

- **Shape.** `open_file`, `open_files`, `pick_folder` and `save_file` are `async` and
  take `&AsyncApp`; call them from `cx.spawn(async move |this, cx| ..)`. The native dialog
  is requested on the UI thread; the future resolves when the user answers.
- **Cancellation.** The user cancelling is `Ok(None)`, not an error. Dropping the future
  does not close an open dialog; it only discards the answer. After awaiting, check that
  the requesting entity still exists (`WeakEntity::update` returns `Err` if it was
  released) before applying the result.
- **Filters.** `ExtensionFilter` extensions are case-insensitive, leading dot optional.
  gpui does not pass filters to the platform dialog, so open selections are filtered after
  the dialog returns: a selection can come back empty (`Some(vec![])`, or `None` from
  `open_file`) even though the user did not cancel. Tell the user why. `save_file` appends
  the first extension of the first non-empty filter when the chosen path has none of the
  accepted ones.
- **Errors.** `Result` is the platform's: a failed dialog request or a dropped response
  channel is `Err`. Show it in the UI (a `MessageBar`) rather than unwrapping.
- **Tests.** On the gpui test platform answer prompts with
  `TestAppContext::simulate_path_prompt_response` / `simulate_new_path_selection`.
- **Platforms.** Dialogs go through gpui's Windows and Linux platform code. Only the test
  platform is covered by automated tests here.

## 4. Support boundaries

The detailed behavior matrix and its open verification work are in [platform-parity.md](platform-parity.md).


- Supported: Windows 10/11 x64 (DirectX 11), Linux x64 X11 and Wayland (wgpu).
- Out of scope: macOS (no Metal paths, no claims) and web/wasm.
- Windows-specific input details (scan codes, UI Automation) are unverified on real
  Windows in this repository's CI; rely on the CHANGELOG entry for each feature.
- Apps import only `fluent-*` crates; `use gpui::` outside them is a layering failure.
