use gpui::{
    App, Application, Bounds, Context, Pixels, Render, VideoTextureId, Window, WindowBounds,
    WindowOptions, canvas, div, prelude::*, px, rgb, size,
};

const FRAME_WIDTH: u32 = 256;
const FRAME_HEIGHT: u32 = 144;
const DISPLAY_WIDTH: Pixels = px(512.0);
const DISPLAY_HEIGHT: Pixels = px(288.0);

struct VideoFrameExample {
    texture: Option<VideoTextureId>,
    allocation_error: Option<String>,
    frame_index: u32,
    frame_data: Vec<u8>,
}

impl VideoFrameExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (texture, allocation_error) =
            match window.alloc_video_texture(FRAME_WIDTH, FRAME_HEIGHT) {
                Ok(texture) => (Some(texture), None),
                Err(error) => (None, Some(error.to_string())),
            };

        cx.on_release_in(window, |this, window, _| {
            if let Some(texture) = this.texture.take() {
                window.free_video_texture(texture);
            }
        })
        .detach();

        Self {
            texture,
            allocation_error,
            frame_index: 0,
            frame_data: vec![0; (FRAME_WIDTH * FRAME_HEIGHT * 4) as usize],
        }
    }

    fn fill_next_frame(&mut self) {
        let t = self.frame_index;

        for y in 0..FRAME_HEIGHT {
            for x in 0..FRAME_WIDTH {
                let offset = ((y * FRAME_WIDTH + x) * 4) as usize;
                let checker = ((x / 16) + (y / 16) + (t / 12)) & 1 == 0;
                self.frame_data[offset] = ((x + t * 2) & 0xff) as u8;
                self.frame_data[offset + 1] = ((y * 2 + t * 3) & 0xff) as u8;
                self.frame_data[offset + 2] = if checker { 240 } else { 48 };
                self.frame_data[offset + 3] = if checker { 255 } else { 160 };
            }
        }

        self.frame_index = self.frame_index.wrapping_add(1);
    }
}

impl Render for VideoFrameExample {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(error) = self.allocation_error.as_ref() {
            return div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .bg(rgb(0x202124))
                .text_color(rgb(0xffffff))
                .child(format!("VideoFrame allocation failed: {error}"));
        }

        let Some(texture) = self.texture else {
            return div().size_full().bg(rgb(0x202124));
        };

        window.request_animation_frame();
        self.fill_next_frame();

        let frame_data = self.frame_data.clone();
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .bg(rgb(0x202124))
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds: Bounds<Pixels>, _, window, _| {
                        if let Err(error) = window.paint_video_frame(texture, bounds, &frame_data) {
                            eprintln!("failed to paint video frame: {error:#}");
                        }
                    },
                )
                .w(DISPLAY_WIDTH)
                .h(DISPLAY_HEIGHT)
                .bg(rgb(0xf3f4f6))
                .border_1()
                .border_color(rgb(0x6b7280)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(760.0), px(520.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| VideoFrameExample::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
