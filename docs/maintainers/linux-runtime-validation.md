# Linux runtime validation

## 2026-11-03 — WI-1438 headless lane

Environment: Linux, sway 1.10.1 with its headless backend at 1280x720,
`WGPU_BACKEND=gl`, and Mesa software rendering (`LIBGL_ALWAYS_SOFTWARE=1`).
This validates the Linux/Wayland wgpu path in software; it is **not** an AMD
Vulkan hardware run. Xvfb/Weston were not installed, so X11 was not runnable.
Screenshots and logs were captured outside the repository.

| Check | Result | Evidence / limitation |
|---|---|---|
| Workspace build | PASS | `cargo build --workspace --all-targets` |
| `hello_world` | PASS | Alive after 4 s; 1280x720 screenshot, 895 colours |
| `gallery` | PASS | Alive after 4 s; screenshot, 7,746 colours |
| `demo_app` | PASS | Alive after 4 s; screenshot, 7,898 colours |
| `widgets` | PASS | Alive after 4 s; screenshot, 5,296 colours |
| `form` | PASS | Alive after 4 s; screenshot, 3,901 colours |
| `screenshots button` | PASS | Alive after 4 s; screenshot, 1,903 colours; fixed-size window did not crash |
| GPUI `video_frame` | PASS (basic) | Alive after 4 s; generated gradient visible; screenshot, 45,507 colours |
| GPUI `scale` | PASS | Alive after 4 s; scaled and unscaled blocks visible; screenshot, 374 colours |
| Video invalid sizes / stale ids | NOT RUN | API checks are covered by tests, but this lane did not add an interactive renderer probe |
| Video alpha vs sprite compositing | NOT RUN | Example is fully opaque |
| Video allocate/upload/free cycles and GPU memory | NOT RUN | No hardware GPU or memory telemetry exposed in this container |
| Overlapping backdrop blur | NOT RUN | `demo_app` currently uses the blur-layer colour token, not `backdrop_blur` |
| Several-blur performance | NOT RUN | No representative blur surface or hardware GPU |
| GNOME/AMD fullscreen regression | NOT RUN | Headless sway software GL cannot reproduce GNOME/AMD Vulkan |

The first Wayland launch exposed a resize/device-resource bug: ordinary quads
panicked while binding a backdrop placeholder after surface invalidation. The
renderer now retains the placeholder across resizes; all eight listed runtime
programs remained alive after the fix.

This evidence does **not** justify “validated on hardware”. The remaining
hardware, X11, video lifecycle/alpha, blur visual/performance, and GNOME/AMD
checks still require a Linux desktop with an AMD Vulkan render node.
