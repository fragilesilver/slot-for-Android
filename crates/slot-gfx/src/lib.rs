mod bezel;
mod draw;
mod fbdev;
mod fbo;
mod gl;
mod grade;
#[cfg(target_os = "macos")]
mod headless;
#[cfg(feature = "host")]
mod host;
mod lcd3x;
mod overlay;
mod pipeline;
mod power;
mod quad;
mod shaders;
mod surface;

pub use bezel::{framed, lifted_rect, screen_rect};
pub use draw::{Draw, TexId};
pub use fbdev::{egl_error, panel_mode, panel_size, FbdevSurface};
pub use fbo::{canvas_size, Compositor, BACKDROP};
pub use grade::{blue_light_gain, colour_grade, BLUE_LIGHT_MAX, COLOUR_GRADES};
#[cfg(target_os = "macos")]
pub use headless::HeadlessSurface;
#[cfg(feature = "host")]
pub use host::HostSurface;
pub use lcd3x::{lcd3x_factors, lcd3x_mask, ScreenEffect};
pub use overlay::{Overlay, Shape};
pub use pipeline::{SCALE, SRC_H, SRC_W, WHOLE_TEXTURE};
pub use power::{screen_brightness, screen_scale, screen_width};
pub use surface::{blit_rect, blit_rect_fit, fit_rect, fit_scale, GfxError, Surface, OUT_H, OUT_W};

/// Loads GL through `surface` and clears the frame to a flat colour (platform bring-up check).
pub fn clear_screen(surface: &mut dyn Surface, rgb: (f32, f32, f32)) {
    gl::load(surface);
    let (w, h) = surface.window_size();
    unsafe {
        ::gl::Viewport(0, 0, w as i32, h as i32);
        ::gl::ClearColor(rgb.0, rgb.1, rgb.2, 1.0);
        ::gl::Clear(::gl::COLOR_BUFFER_BIT);
    }
}
