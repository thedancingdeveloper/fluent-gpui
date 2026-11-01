//! [`VideoSurface`]: shows caller-supplied RGBA8 video frames on a standalone GPU texture.
//!
//! # Model
//!
//! The caller owns the frame source (a decoder, a remote desktop stream, a generator) and hands
//! frames to the surface with [`VideoSurface::submit_frame`]. A frame is tightly packed RGBA8
//! (`R, G, B, A` per pixel, rows top to bottom, no padding), at most
//! [`MAX_VIDEO_FRAME_DIMENSION`] pixels per side. When only part of the picture changed,
//! [`VideoSurface::submit_region`] patches a rectangle of the last frame instead of sending a
//! whole one; the surface keeps a CPU copy of the frame for this.
//!
//! The surface owns the texture. It allocates it on the first frame, frees and reallocates it
//! when the frame size changes, and frees it when the view is released. Frames are uploaded on
//! the next paint, so submitting several frames between paints uploads only the last.
//!
//! # Scaling
//!
//! [`VideoScaling::Fit`] scales the frame to the largest size that fits the bounds, keeping its
//! aspect ratio. [`VideoScaling::Native`] shows one frame pixel per device pixel, snapped to the
//! device pixel grid, so the picture is sampled without filtering; a frame larger than the bounds
//! is centred and clipped. The bars left around the frame are painted as a letterbox.
//!
//! gpui's video sampler is linear, so a Fit frame that is scaled is filtered; true
//! nearest-neighbour scaling needs a renderer change and is not available yet.
//!
//! # Layout and input
//!
//! After each paint, [`VideoSurface::layout`] returns a [`VideoLayout`]: the widget bounds, the
//! rectangle the frame occupies, and the frame size. Use [`VideoLayout::to_frame`] to turn a
//! window position (from an [`InputCapture`](crate::InputCapture) pointer event, say) into frame
//! pixel coordinates. When the widget's size in device pixels changes the view emits
//! [`VideoSurfaceEvent::Resized`], so a remote source can be asked for frames at that size.
//!
//! # Tests and the placeholder backend
//!
//! gpui's test platform keeps video textures but does not render them. Build the surface with
//! [`VideoSurface::placeholder`] in tests: it never touches the window's video textures, paints
//! a flat placeholder where the frame would be, and lays out exactly as the GPU backend does.
//! Attach a [`VideoRecorder`] (to either backend) to assert what the surface was asked to show.
//!
//! # Accessibility
//!
//! The element has the `Video` role and the label set with [`VideoSurface::label`].

use std::{cell::RefCell, fmt, rc::Rc};

use fluent_core::ThemeProvider as _;
use gpui::{
    canvas, div, fill, point, prelude::*, px, size, AnyWindowHandle, App, Bounds, ContentMask,
    Context, Corners, EventEmitter, Hsla, IntoElement, Pixels, Point, Render, Role, SharedString,
    Size, Stateful, Subscription, VideoTextureId, WeakEntity, Window,
};

/// The largest frame side, in pixels, that gpui accepts for a video texture.
pub const MAX_VIDEO_FRAME_DIMENSION: u32 = 8192;

/// How a frame is scaled into the surface bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VideoScaling {
    /// The largest size that fits the bounds, keeping the aspect ratio.
    #[default]
    Fit,
    /// One frame pixel per device pixel, centred and clipped to the bounds.
    Native,
}

/// Something the surface reports to its owner.
#[derive(Clone, Debug, PartialEq)]
pub enum VideoSurfaceEvent {
    /// The widget's size changed. `device_width` and `device_height` are in device pixels, the
    /// frame size that fills the widget at [`VideoScaling::Native`].
    Resized {
        /// The new bounds, in window coordinates.
        bounds: Bounds<Pixels>,
        /// Width in device pixels.
        device_width: u32,
        /// Height in device pixels.
        device_height: u32,
    },
}

/// Why a frame was refused. The previous frame stays on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VideoFrameError {
    /// A side is zero or larger than [`MAX_VIDEO_FRAME_DIMENSION`].
    BadSize {
        /// Frame width.
        width: u32,
        /// Frame height.
        height: u32,
    },
    /// The byte count is not `width * height * 4`.
    BadLength {
        /// Bytes expected.
        expected: usize,
        /// Bytes given.
        actual: usize,
    },
    /// A region was submitted before any frame, or lies outside the frame.
    RegionOutOfBounds,
}

impl fmt::Display for VideoFrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSize { width, height } => write!(
                f,
                "frame {width}x{height} is outside 1..={MAX_VIDEO_FRAME_DIMENSION} per side"
            ),
            Self::BadLength { expected, actual } => {
                write!(f, "frame has {actual} bytes, expected {expected}")
            }
            Self::RegionOutOfBounds => f.write_str("region is outside the current frame"),
        }
    }
}

impl std::error::Error for VideoFrameError {}

/// A rectangle of frame pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameRect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// Where the frame sits inside the surface, from the last paint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoLayout {
    /// The widget bounds, in window coordinates.
    pub bounds: Bounds<Pixels>,
    /// The rectangle the frame is drawn into, in window coordinates. In
    /// [`VideoScaling::Native`] it can extend past `bounds`.
    pub content: Bounds<Pixels>,
    /// The frame size in pixels.
    pub frame: Size<u32>,
}

impl VideoLayout {
    /// Lay out a `frame`-sized picture in `bounds` with the given scaling and device scale
    /// factor.
    pub fn compute(
        bounds: Bounds<Pixels>,
        frame: Size<u32>,
        scaling: VideoScaling,
        scale_factor: f32,
    ) -> Self {
        let scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let (fw, fh) = (frame.width as f32, frame.height as f32);
        let (bw, bh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let (w, h) = match scaling {
            _ if fw <= 0.0 || fh <= 0.0 => (0.0, 0.0),
            VideoScaling::Fit => {
                let k = (bw / fw).min(bh / fh).max(0.0);
                (fw * k, fh * k)
            }
            VideoScaling::Native => (fw / scale_factor, fh / scale_factor),
        };
        let mut x = f32::from(bounds.origin.x) + (bw - w) / 2.0;
        let mut y = f32::from(bounds.origin.y) + (bh - h) / 2.0;
        if scaling == VideoScaling::Native {
            // Land each frame pixel on a whole device pixel so sampling is exact.
            x = (x * scale_factor).round() / scale_factor;
            y = (y * scale_factor).round() / scale_factor;
        }
        Self {
            bounds,
            content: Bounds::new(point(px(x), px(y)), size(px(w), px(h))),
            frame,
        }
    }

    /// The frame pixel under a window position, or `None` outside the frame (in the letterbox,
    /// or clipped away). Coordinates are fractional: `(0.0, 0.0)` is the top-left corner of the
    /// frame and `(width, height)` the bottom-right.
    pub fn to_frame(&self, position: Point<Pixels>) -> Option<Point<f32>> {
        if !self.bounds.contains(&position) {
            return None;
        }
        let p = self.to_frame_clamped(position)?;
        let inside = self.content.contains(&position);
        inside.then_some(p)
    }

    /// Like [`Self::to_frame`], but clamps positions outside the frame to its edge. Useful for
    /// a drag that leaves the picture. `None` only when no frame has been shown.
    pub fn to_frame_clamped(&self, position: Point<Pixels>) -> Option<Point<f32>> {
        let cw = f32::from(self.content.size.width);
        let ch = f32::from(self.content.size.height);
        if cw <= 0.0 || ch <= 0.0 {
            return None;
        }
        let (fw, fh) = (self.frame.width as f32, self.frame.height as f32);
        let x = (f32::from(position.x - self.content.origin.x) / cw * fw).clamp(0.0, fw);
        let y = (f32::from(position.y - self.content.origin.y) / ch * fh).clamp(0.0, fh);
        Some(Point { x, y })
    }
}

/// One thing a [`VideoSurface`] did, as logged by a [`VideoRecorder`].
#[derive(Clone, Debug, PartialEq)]
pub enum VideoRecord {
    /// A texture of this size was allocated.
    Alloc {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
    },
    /// A frame was uploaded.
    Upload {
        /// Width.
        width: u32,
        /// Height.
        height: u32,
        /// The pixel bytes uploaded.
        pixels: Rc<[u8]>,
        /// The regions changed since the previous upload; empty for a whole new frame.
        dirty: Vec<FrameRect>,
    },
    /// The texture was freed.
    Free,
    /// The frame (or, with the placeholder backend, the placeholder) was painted.
    Paint {
        /// Where the frame was drawn.
        content: Bounds<Pixels>,
    },
}

/// A shared log of what a [`VideoSurface`] did. Clone it, attach one clone with
/// [`VideoSurface::with_recorder`], and read the other.
#[derive(Clone, Default)]
pub struct VideoRecorder(Rc<RefCell<Vec<VideoRecord>>>);

impl VideoRecorder {
    /// A new, empty recorder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Everything recorded so far, oldest first.
    pub fn records(&self) -> Vec<VideoRecord> {
        self.0.borrow().clone()
    }

    /// The last uploaded frame: `(width, height, pixels)`.
    pub fn last_upload(&self) -> Option<(u32, u32, Rc<[u8]>)> {
        self.0.borrow().iter().rev().find_map(|r| match r {
            VideoRecord::Upload {
                width,
                height,
                pixels,
                ..
            } => Some((*width, *height, pixels.clone())),
            _ => None,
        })
    }

    /// Forget everything recorded so far.
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    fn push(&self, record: VideoRecord) {
        self.0.borrow_mut().push(record);
    }
}

impl fmt::Debug for VideoRecorder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VideoRecorder")
            .field(&self.0.borrow().len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    Gpu,
    Placeholder,
}

/// The texture (or, for the placeholder backend, the pretend texture) currently allocated.
#[derive(Clone, Copy, Debug)]
struct Texture {
    id: Option<VideoTextureId>,
    width: u32,
    height: u32,
}

/// State shared between the view and its paint callback, which has the window.
struct Inner {
    backend: Backend,
    recorder: Option<VideoRecorder>,
    /// The latest frame, kept for region patches.
    frame: Option<(u32, u32, Vec<u8>)>,
    /// The frame needs uploading, with these dirty regions (empty means all of it).
    pending: Option<Vec<FrameRect>>,
    texture: Option<Texture>,
    layout: Option<VideoLayout>,
    device_size: Option<(u32, u32)>,
    /// The window the texture lives in, from the last paint.
    window: Option<AnyWindowHandle>,
}

impl Inner {
    fn record(&self, record: impl FnOnce() -> VideoRecord) {
        if let Some(recorder) = &self.recorder {
            recorder.push(record());
        }
    }

    fn free(&mut self, window: Option<&mut Window>) {
        let Some(texture) = self.texture.take() else {
            return;
        };
        if let (Some(id), Some(window)) = (texture.id, window) {
            window.free_video_texture(id);
        }
        self.record(|| VideoRecord::Free);
    }

    /// Allocate or reallocate as needed and upload a pending frame. Runs during paint.
    fn sync(&mut self, window: &mut Window) {
        let Some(dirty) = self.pending.take() else {
            return;
        };
        let Some((width, height)) = self.frame.as_ref().map(|(w, h, _)| (*w, *h)) else {
            return;
        };
        let stale = self
            .texture
            .is_some_and(|t| t.width != width || t.height != height);
        if stale {
            self.free(Some(window));
        }
        if self.texture.is_none() {
            let id = match self.backend {
                Backend::Gpu => Some(window.alloc_video_texture(width, height)),
                Backend::Placeholder => None,
            };
            self.texture = Some(Texture { id, width, height });
            self.record(|| VideoRecord::Alloc { width, height });
        }
        let Some((_, _, pixels)) = &self.frame else {
            return;
        };
        let id = self.texture.and_then(|t| t.id);
        let uploaded = match id {
            Some(id) => window.upload_video_frame(id, pixels.clone()).is_ok(),
            None => true,
        };
        if uploaded {
            self.record(|| VideoRecord::Upload {
                width,
                height,
                pixels: pixels.as_slice().into(),
                dirty,
            });
        }
    }
}

/// Shows caller-supplied RGBA8 frames, scaled to fit or 1:1. See the [module docs](self).
pub struct VideoSurface {
    inner: Rc<RefCell<Inner>>,
    scaling: VideoScaling,
    label: SharedString,
    _release: Subscription,
}

impl EventEmitter<VideoSurfaceEvent> for VideoSurface {}

impl VideoSurface {
    /// A surface that paints frames on a GPU texture.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::with_backend(Backend::Gpu, cx)
    }

    /// A surface that never touches video textures and paints a placeholder instead; see the
    /// [module docs](self#tests-and-the-placeholder-backend).
    pub fn placeholder(cx: &mut Context<Self>) -> Self {
        Self::with_backend(Backend::Placeholder, cx)
    }

    fn with_backend(backend: Backend, cx: &mut Context<Self>) -> Self {
        let inner = Rc::new(RefCell::new(Inner {
            backend,
            recorder: None,
            frame: None,
            pending: None,
            texture: None,
            layout: None,
            device_size: None,
            window: None,
        }));
        // Free through a deferred window update: the window may be mid-update when the view is
        // released. A closed window has dropped its textures already.
        let release = cx.on_release(|this: &mut Self, cx| {
            let inner = this.inner.clone();
            let handle = inner.borrow().window;
            match handle {
                Some(handle) => cx.defer(move |cx| {
                    let freed = handle.update(cx, |_, window, _| {
                        inner.borrow_mut().free(Some(window));
                    });
                    if freed.is_err() {
                        inner.borrow_mut().free(None);
                    }
                }),
                None => inner.borrow_mut().free(None),
            }
        });
        Self {
            inner,
            scaling: VideoScaling::default(),
            label: "Video".into(),
            _release: release,
        }
    }

    /// Log what the surface does into `recorder`.
    pub fn with_recorder(self, recorder: VideoRecorder) -> Self {
        self.inner.borrow_mut().recorder = Some(recorder);
        self
    }

    /// Set the scaling mode.
    pub fn set_scaling(&mut self, scaling: VideoScaling, cx: &mut Context<Self>) {
        if self.scaling != scaling {
            self.scaling = scaling;
            cx.notify();
        }
    }

    /// The scaling mode.
    pub fn scaling(&self) -> VideoScaling {
        self.scaling
    }

    /// Set the accessible name (default "Video").
    pub fn label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.label = label.into();
        cx.notify();
    }

    /// Show a whole new frame of tightly packed RGBA8 pixels. It is uploaded on the next paint.
    pub fn submit_frame(
        &mut self,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        cx: &mut Context<Self>,
    ) -> Result<(), VideoFrameError> {
        check_frame(width, height, pixels.len())?;
        let mut inner = self.inner.borrow_mut();
        inner.frame = Some((width, height, pixels));
        inner.pending = Some(Vec::new());
        drop(inner);
        cx.notify();
        Ok(())
    }

    /// Replace a rectangle of the current frame with tightly packed RGBA8 `pixels`
    /// (`rect.width * rect.height * 4` bytes).
    pub fn submit_region(
        &mut self,
        rect: FrameRect,
        pixels: &[u8],
        cx: &mut Context<Self>,
    ) -> Result<(), VideoFrameError> {
        let mut inner = self.inner.borrow_mut();
        let Some((width, height, frame)) = inner.frame.as_mut() else {
            return Err(VideoFrameError::RegionOutOfBounds);
        };
        let fits = rect.x.checked_add(rect.width).is_some_and(|r| r <= *width)
            && rect
                .y
                .checked_add(rect.height)
                .is_some_and(|b| b <= *height);
        if !fits {
            return Err(VideoFrameError::RegionOutOfBounds);
        }
        let row = rect.width as usize * 4;
        let expected = row * rect.height as usize;
        if pixels.len() != expected {
            return Err(VideoFrameError::BadLength {
                expected,
                actual: pixels.len(),
            });
        }
        let stride = *width as usize * 4;
        for (i, src) in pixels.chunks_exact(row.max(1)).enumerate() {
            let start = (rect.y as usize + i) * stride + rect.x as usize * 4;
            if let Some(dst) = frame.get_mut(start..start + row) {
                dst.copy_from_slice(src);
            }
        }
        // A whole-frame upload already pending covers this region too.
        match inner.pending.as_mut() {
            Some(dirty) if dirty.is_empty() => {}
            Some(dirty) => dirty.push(rect),
            None => inner.pending = Some(vec![rect]),
        }
        drop(inner);
        cx.notify();
        Ok(())
    }

    /// Stop showing a frame and free the texture.
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut inner = self.inner.borrow_mut();
        inner.free(Some(window));
        inner.frame = None;
        inner.pending = None;
        drop(inner);
        cx.notify();
    }

    /// The size of the frame being shown.
    pub fn frame_size(&self) -> Option<Size<u32>> {
        self.inner
            .borrow()
            .frame
            .as_ref()
            .map(|(w, h, _)| size(*w, *h))
    }

    /// Where the frame sat at the last paint; `None` before the first paint.
    pub fn layout(&self) -> Option<VideoLayout> {
        self.inner.borrow().layout
    }

    /// Map a window position to frame pixel coordinates; see [`VideoLayout::to_frame`].
    pub fn to_frame(&self, position: Point<Pixels>) -> Option<Point<f32>> {
        self.layout()?.to_frame(position)
    }

    fn paint(
        inner: &Rc<RefCell<Inner>>,
        weak: &WeakEntity<Self>,
        scaling: VideoScaling,
        colors: (Hsla, Hsla),
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (letterbox, placeholder) = colors;
        let scale_factor = window.scale_factor();
        let mut state = inner.borrow_mut();
        state.window = Some(window.window_handle());
        state.sync(window);

        let device = (
            (f32::from(bounds.size.width) * scale_factor)
                .round()
                .max(0.0) as u32,
            (f32::from(bounds.size.height) * scale_factor)
                .round()
                .max(0.0) as u32,
        );
        let resized = state.device_size != Some(device);
        state.device_size = Some(device);

        let frame = state.texture.map(|t| size(t.width, t.height));
        let layout = VideoLayout::compute(bounds, frame.unwrap_or_default(), scaling, scale_factor);
        state.layout = Some(layout);

        window.paint_quad(fill(bounds, letterbox));
        if let Some(texture) = state.texture {
            window.with_content_mask(Some(ContentMask { bounds }), |window| match texture.id {
                Some(id) => window.paint_video_frame(layout.content, Corners::default(), id),
                None => window.paint_quad(fill(layout.content, placeholder)),
            });
            state.record(|| VideoRecord::Paint {
                content: layout.content,
            });
        }
        drop(state);

        if resized {
            let event = VideoSurfaceEvent::Resized {
                bounds,
                device_width: device.0,
                device_height: device.1,
            };
            weak.update(cx, |_, cx| cx.emit(event)).ok();
        }
    }
}

fn check_frame(width: u32, height: u32, len: usize) -> Result<(), VideoFrameError> {
    let side_ok = |s: u32| (1..=MAX_VIDEO_FRAME_DIMENSION).contains(&s);
    if !side_ok(width) || !side_ok(height) {
        return Err(VideoFrameError::BadSize { width, height });
    }
    let expected = width as usize * height as usize * 4;
    if len != expected {
        return Err(VideoFrameError::BadLength {
            expected,
            actual: len,
        });
    }
    Ok(())
}

/// The element carrying the accessibility semantics: a video named by `label`.
fn a11y_base(label: SharedString) -> Stateful<gpui::Div> {
    div()
        .id("fluent-video-surface")
        .role(Role::Video)
        .aria_label(label)
}

impl Render for VideoSurface {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let paint_colors = (gpui::black(), colors.surface_dim);
        let inner = self.inner.clone();
        let weak = cx.weak_entity();
        let scaling = self.scaling;
        let surface = canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                Self::paint(&inner, &weak, scaling, paint_colors, bounds, window, cx)
            },
        )
        .size_full();

        a11y_base(self.label.clone())
            .size_full()
            .overflow_hidden()
            .child(surface)
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, VisualTestContext};
    use gpui::{Entity, TestAppContext};

    use super::*;

    struct Host {
        surface: Option<Entity<VideoSurface>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(
                div()
                    .w(px(400.0))
                    .h(px(200.0))
                    .children(self.surface.clone()),
            )
        }
    }

    type Events = Rc<RefCell<Vec<VideoSurfaceEvent>>>;

    fn setup(
        cx: &mut TestAppContext,
        placeholder: bool,
    ) -> (
        Entity<Host>,
        Entity<VideoSurface>,
        VideoRecorder,
        Events,
        &mut VisualTestContext,
    ) {
        let recorder = VideoRecorder::new();
        let events: Events = Rc::default();
        let (rec, sink) = (recorder.clone(), events.clone());
        let (host, vcx) = open_fluent_test_window(cx, move |_, cx| {
            let surface = cx.new(|cx| {
                let surface = if placeholder {
                    VideoSurface::placeholder(cx)
                } else {
                    VideoSurface::new(cx)
                };
                surface.with_recorder(rec)
            });
            cx.subscribe(&surface, move |_, _, event: &VideoSurfaceEvent, _| {
                sink.borrow_mut().push(event.clone());
            })
            .detach();
            Host {
                surface: Some(surface),
            }
        });
        let surface = host.read_with(vcx, |h, _| h.surface.clone().expect("surface"));
        draw(vcx);
        (host, surface, recorder, events, vcx)
    }

    fn draw(vcx: &mut VisualTestContext) {
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        vcx.run_until_parked();
    }

    fn submit(surface: &Entity<VideoSurface>, vcx: &mut VisualTestContext, w: u32, h: u32) {
        surface
            .update(vcx, |s, cx| {
                s.submit_frame(w, h, vec![7; (w * h * 4) as usize], cx)
            })
            .expect("valid frame");
    }

    /// The kinds of record, with repeats collapsed: painting a video frame requests another
    /// animation frame, so one draw can paint more than once.
    fn kinds(recorder: &VideoRecorder) -> Vec<&'static str> {
        let mut kinds: Vec<&'static str> = recorder
            .records()
            .iter()
            .map(|r| match r {
                VideoRecord::Alloc { .. } => "alloc",
                VideoRecord::Upload { .. } => "upload",
                VideoRecord::Free => "free",
                VideoRecord::Paint { .. } => "paint",
            })
            .collect();
        kinds.dedup();
        kinds
    }

    fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    #[test]
    fn fit_letterboxes_and_maps_coordinates() {
        let layout = VideoLayout::compute(
            bounds(0.0, 0.0, 400.0, 200.0),
            size(200, 200),
            VideoScaling::Fit,
            1.0,
        );
        assert_eq!(layout.content, bounds(100.0, 0.0, 200.0, 200.0));
        assert_eq!(
            layout.to_frame(point(px(50.0), px(100.0))),
            None,
            "letterbox"
        );
        assert_eq!(
            layout.to_frame(point(px(200.0), px(50.0))),
            Some(Point { x: 100.0, y: 50.0 })
        );
        assert_eq!(
            layout.to_frame_clamped(point(px(10.0), px(50.0))),
            Some(Point { x: 0.0, y: 50.0 })
        );
    }

    #[test]
    fn native_is_one_to_one_in_device_pixels_and_snapped() {
        let layout = VideoLayout::compute(
            bounds(0.0, 0.0, 101.0, 100.0),
            size(100, 50),
            VideoScaling::Native,
            2.0,
        );
        // 100x50 device pixels at 2x is 50x25 logical, centred at 25.5 and snapped to 0.5 steps.
        assert_eq!(layout.content.size, size(px(50.0), px(25.0)));
        assert_eq!(layout.content.origin, point(px(25.5), px(37.5)));
        assert_eq!(
            layout.to_frame(point(px(25.5 + 10.0), px(37.5 + 5.0))),
            Some(Point { x: 20.0, y: 10.0 })
        );
        // A frame bigger than the bounds overflows and positions past the bounds are refused.
        let big = VideoLayout::compute(
            bounds(0.0, 0.0, 10.0, 10.0),
            size(40, 40),
            VideoScaling::Native,
            1.0,
        );
        assert_eq!(big.content, bounds(-15.0, -15.0, 40.0, 40.0));
        assert_eq!(big.to_frame(point(px(-1.0), px(5.0))), None);
        assert_eq!(
            big.to_frame(point(px(0.0), px(0.0))),
            Some(Point { x: 15.0, y: 15.0 })
        );
    }

    #[test]
    fn rejects_bad_frames() {
        assert_eq!(
            check_frame(0, 4, 0),
            Err(VideoFrameError::BadSize {
                width: 0,
                height: 4
            })
        );
        assert!(check_frame(MAX_VIDEO_FRAME_DIMENSION + 1, 1, 0).is_err());
        assert_eq!(
            check_frame(2, 2, 15),
            Err(VideoFrameError::BadLength {
                expected: 16,
                actual: 15
            })
        );
        assert_eq!(check_frame(2, 2, 16), Ok(()));
    }

    #[test]
    fn exposes_video_role_and_label() {
        let element = a11y_base("Remote desktop".into());
        assert_eq!(element.a11y_role(), Some(Role::Video));
        let mut node = gpui::accesskit::Node::new(Role::Video);
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Remote desktop"));
    }

    #[gpui::test]
    fn gpu_backend_allocs_reallocs_and_frees(cx: &mut TestAppContext) {
        let (host, surface, recorder, _, vcx) = setup(cx, false);
        assert!(recorder.records().is_empty(), "nothing before a frame");

        submit(&surface, vcx, 4, 2);
        draw(vcx);
        assert_eq!(kinds(&recorder), ["alloc", "upload", "paint"]);
        let id = surface.read_with(vcx, |s, _| s.inner.borrow().texture.and_then(|t| t.id));
        assert!(id.is_some(), "a real texture on the GPU backend");

        // Same size: upload into the same texture.
        recorder.clear();
        submit(&surface, vcx, 4, 2);
        draw(vcx);
        assert_eq!(kinds(&recorder), ["upload", "paint"]);

        // New size: free and reallocate.
        recorder.clear();
        submit(&surface, vcx, 8, 8);
        draw(vcx);
        assert_eq!(kinds(&recorder), ["free", "alloc", "upload", "paint"]);
        assert_eq!(
            recorder.records()[1],
            VideoRecord::Alloc {
                width: 8,
                height: 8
            }
        );

        // Dropping the view frees the texture.
        recorder.clear();
        host.update(vcx, |h, cx| {
            h.surface = None;
            cx.notify();
        });
        drop(surface);
        draw(vcx);
        assert_eq!(kinds(&recorder), ["free"]);
    }

    #[gpui::test]
    fn placeholder_records_frames_without_textures(cx: &mut TestAppContext) {
        let (_host, surface, recorder, _, vcx) = setup(cx, true);
        let pixels: Vec<u8> = (0..16).collect();
        surface
            .update(vcx, |s, cx| s.submit_frame(2, 2, pixels.clone(), cx))
            .expect("valid");
        draw(vcx);
        assert_eq!(
            recorder.last_upload(),
            Some((2, 2, pixels.as_slice().into()))
        );
        let texture = surface.read_with(vcx, |s, _| s.inner.borrow().texture);
        assert!(texture.is_some_and(|t| t.id.is_none()), "no GPU texture");
        assert_eq!(
            recorder.records().last(),
            Some(&VideoRecord::Paint {
                content: bounds(100.0, 0.0, 200.0, 200.0)
            })
        );

        // A region patch updates the kept frame and reports the dirty rect.
        let rect = FrameRect {
            x: 1,
            y: 1,
            width: 1,
            height: 1,
        };
        surface
            .update(vcx, |s, cx| s.submit_region(rect, &[9, 9, 9, 9], cx))
            .expect("in bounds");
        draw(vcx);
        let last = recorder.records().into_iter().rev().find_map(|r| match r {
            VideoRecord::Upload { pixels, dirty, .. } => Some((pixels, dirty)),
            _ => None,
        });
        let (patched, dirty) = last.expect("uploaded");
        assert_eq!(dirty, [rect]);
        assert_eq!(&patched[12..16], &[9, 9, 9, 9]);
        assert_eq!(&patched[0..12], &pixels[0..12]);

        let outside = FrameRect {
            x: 2,
            y: 0,
            width: 1,
            height: 1,
        };
        let err = surface.update(vcx, |s, cx| s.submit_region(outside, &[0; 4], cx));
        assert_eq!(err, Err(VideoFrameError::RegionOutOfBounds));
    }

    #[gpui::test]
    fn reports_resize_and_maps_pointer(cx: &mut TestAppContext) {
        let (_host, surface, _, events, vcx) = setup(cx, true);
        submit(&surface, vcx, 100, 100);
        draw(vcx);
        let resized = events.borrow().iter().any(|e| {
            matches!(e, VideoSurfaceEvent::Resized { device_width, device_height, .. }
                if *device_width > 0 && *device_height > 0)
        });
        assert!(resized, "resize reported: {:?}", events.borrow());
        let count = events.borrow().len();
        draw(vcx);
        assert_eq!(
            events.borrow().len(),
            count,
            "no repeat without a size change"
        );

        let mapped = surface.read_with(vcx, |s, _| s.to_frame(point(px(200.0), px(100.0))));
        assert_eq!(mapped, Some(Point { x: 50.0, y: 50.0 }));
    }
}
