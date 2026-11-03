# Windows runtime validation (WI-1439)

Host: `desktop-tlk1gff` (Windows, GPU, console session 1). Build: `main` 2b77306,
`cargo build --workspace --all-targets` passed. Each example was launched in the
interactive session through a one-off scheduled task (`/IT /RU`), then the task
was deleted. Nothing persistent was left behind.

| Example | Launches in session 1 | Alive after 10 s | Window title | Pixel check |
|---|---|---|---|---|
| hello_world | yes | yes | Hello FluentGUI | inconclusive |
| demo_app | yes | yes | FluentGUI Demo - Project Tracker | inconclusive |
| gallery | yes | yes | FluentGUI Gallery | inconclusive |
| video_frame (`vendor/zed/crates/gpui`) | yes | yes | (none) | inconclusive |

Over plain SSH the same binaries panic with `cannot open windows while headless`
(expected: no desktop in the SSH session).

## Not verified
- Screenshots: every capture (full screen `CopyFromScreen`, window-rect crop and
  `PrintWindow`) came back uniformly white, even with a correct window rect.
  The RDP console or DirectX flip-model presentation is likely not capturable
  this way, so rendered output is unconfirmed.
- Window chrome (move/resize/zoom/minimise), VideoFrame items 2 and 7, and the
  backdrop-blur fallback need a human at the console or a capture path that
  reads the swap chain (for example DXGI Desktop Duplication).

## Note
`cargo run -p gpui --example video_frame` does not resolve from the workspace
root; the example lives in `vendor/zed/crates/gpui` and builds from there.
