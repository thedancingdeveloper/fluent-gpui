//! Paints a generated RGBA8 frame into a standalone video texture every frame.

use gpui::{
    App, Bounds, Context, Corners, VideoTextureId, Window, WindowBounds, WindowOptions, canvas,
    div, prelude::*, px, rgb, size,
};
use gpui_platform::application;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

struct VideoFrameExample {
    texture: Option<VideoTextureId>,
    frame: u32,
}

impl VideoFrameExample {
    /// A moving gradient with a sweeping bar, in packed RGBA8.
    fn next_frame(&mut self) -> Vec<u8> {
        self.frame = self.frame.wrapping_add(1);
        let bar = self.frame % WIDTH;
        let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if x.abs_diff(bar) < 4 {
                    pixels.extend_from_slice(&[255, 255, 255, 255]);
                } else {
                    let r = ((x + self.frame) % 256) as u8;
                    let g = ((y * 255) / HEIGHT) as u8;
                    pixels.extend_from_slice(&[r, g, 160, 255]);
                }
            }
        }
        pixels
    }
}

impl Render for VideoFrameExample {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let texture = *self
            .texture
            .get_or_insert_with(|| window.alloc_video_texture(WIDTH, HEIGHT));
        if let Err(error) = window.upload_video_frame(texture, self.next_frame()) {
            eprintln!("video frame upload failed: {error}");
        }

        div()
            .flex()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0x202020))
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, window, _| {
                        window.paint_video_frame(bounds, Corners::all(px(12.)), texture);
                    },
                )
                .w(px(WIDTH as f32 * 2.))
                .h(px(HEIGHT as f32 * 2.)),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(500.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| VideoFrameExample {
                    texture: None,
                    frame: 0,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
