//! Flat translucent shapes drawn over the finished frame (touch controls).
use crate::quad::Quad;
use crate::shaders::{program, RECT_VERT, SPRITE_FRAG};
use crate::surface::{GfxError, Surface};

const FRAG: &str = r#"
precision mediump float;
uniform vec4 u_color;
uniform float u_round;
varying vec2 v_uv;
void main() {
    float a = 1.0;
    if (u_round > 0.5) {
        a = 1.0 - smoothstep(0.46, 0.5, length(v_uv - 0.5));
    }
    FRAG_COLOR = vec4(u_color.rgb, u_color.a * a);
}
"#;

#[derive(Copy, Clone, Debug)]
pub struct Shape {
    /// x, y, w, h in window pixels, origin top-left.
    pub rect: [f32; 4],
    pub round: bool,
    pub rgba: [f32; 4],
    /// A texture from [`Overlay::add_label`]: drawn as a sprite tinted by `rgba`.
    pub label: Option<usize>,
}

pub struct Overlay {
    prog: gl::types::GLuint,
    sprite: gl::types::GLuint,
    sprite_rect: gl::types::GLint,
    sprite_target: gl::types::GLint,
    sprite_colour: gl::types::GLint,
    sprite_tex: gl::types::GLint,
    labels: Vec<gl::types::GLuint>,
    quad: Quad,
    rect: gl::types::GLint,
    target: gl::types::GLint,
    color: gl::types::GLint,
    round: gl::types::GLint,
}

impl Overlay {
    pub fn new(surface: &dyn Surface) -> Result<Self, GfxError> {
        crate::gl::load(surface);
        let prog = program(RECT_VERT, FRAG)?;
        let sprite = program(RECT_VERT, SPRITE_FRAG)?;
        let loc = |n| crate::gl::uniform_location(prog, n);
        let sloc = |n| crate::gl::uniform_location(sprite, n);
        Ok(Overlay {
            prog,
            sprite,
            sprite_rect: sloc("u_rect"),
            sprite_target: sloc("u_target"),
            sprite_colour: sloc("u_colour"),
            sprite_tex: sloc("u_tex"),
            labels: Vec::new(),
            quad: Quad::new(),
            rect: loc("u_rect"),
            target: loc("u_target"),
            color: loc("u_color"),
            round: loc("u_round"),
        })
    }

    /// Uploads straight-alpha RGBA pixels as a label texture and returns its id.
    pub fn add_label(&mut self, w: u32, h: u32, rgba: &[u8]) -> usize {
        let tex = crate::gl::texture(w, h, gl::LINEAR, gl::CLAMP_TO_EDGE, gl::RGBA, Some(rgba));
        self.labels.push(tex);
        self.labels.len() - 1
    }

    /// Frees every label (the layout changed and they are about to be rebuilt).
    pub fn clear_labels(&mut self) {
        unsafe {
            for tex in self.labels.drain(..) {
                gl::DeleteTextures(1, &tex);
            }
        }
    }

    pub fn draw(&self, window: (u32, u32), shapes: &[Shape]) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, window.0 as i32, window.1 as i32);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            for s in shapes {
                match s.label.and_then(|i| self.labels.get(i)) {
                    Some(&tex) => {
                        gl::UseProgram(self.sprite);
                        gl::Uniform2f(self.sprite_target, window.0 as f32, window.1 as f32);
                        gl::ActiveTexture(gl::TEXTURE0);
                        gl::BindTexture(gl::TEXTURE_2D, tex);
                        gl::Uniform1i(self.sprite_tex, 0);
                        gl::Uniform4f(
                            self.sprite_rect,
                            s.rect[0],
                            s.rect[1],
                            s.rect[2],
                            s.rect[3],
                        );
                        gl::Uniform4f(
                            self.sprite_colour,
                            s.rgba[0],
                            s.rgba[1],
                            s.rgba[2],
                            s.rgba[3],
                        );
                    }
                    None => {
                        gl::UseProgram(self.prog);
                        gl::Uniform2f(self.target, window.0 as f32, window.1 as f32);
                        gl::Uniform4f(self.rect, s.rect[0], s.rect[1], s.rect[2], s.rect[3]);
                        gl::Uniform4f(self.color, s.rgba[0], s.rgba[1], s.rgba[2], s.rgba[3]);
                        gl::Uniform1f(self.round, if s.round { 1.0 } else { 0.0 });
                    }
                }
                self.quad.draw();
            }
            gl::Disable(gl::BLEND);
        }
    }
}
