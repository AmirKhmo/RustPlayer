//! Renders mpv frames into an OpenGL FBO, then blits them into the egui video rect.
use crate::mpv::Player;
use anyhow::{anyhow, Result};
use eframe::{
    egui,
    glow::{self, HasContext},
};
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use std::{
    ffi::{c_void, CStr, CString},
    sync::Mutex,
};

type Loader = dyn Fn(&CStr) -> *const c_void;

/// Only used while mpv loads GL symbols during RenderContext creation.
pub struct ProcLoader(*const Loader);

fn get_proc_address(l: &ProcLoader, name: &str) -> *mut c_void {
    match CString::new(name) {
        Ok(c) => unsafe { (*l.0)(&c) as *mut c_void },
        Err(_) => std::ptr::null_mut(),
    }
}

pub struct VideoRenderer {
    mpv_gl: RenderContext,
    fbo: Option<glow::Framebuffer>,
    tex: Option<glow::Texture>,
    size: (i32, i32),
}

/// egui paint callbacks must be Send + Sync; everything here runs on the GL (main) thread.
pub struct Shared(pub Mutex<Option<VideoRenderer>>);
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}

impl VideoRenderer {
    pub fn new(
        player: &Player,
        loader: &dyn Fn(&CStr) -> *const c_void,
        egui_ctx: egui::Context,
    ) -> Result<Self> {
        let raw: *const (dyn Fn(&CStr) -> *const c_void + '_) = loader;
        let raw: *const Loader = unsafe { std::mem::transmute(raw) };
        let mut handle = player.mpv.ctx;
        let mut mpv_gl = RenderContext::new(
            unsafe { handle.as_mut() },
            vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address,
                    ctx: ProcLoader(raw),
                }),
            ],
        )
        .map_err(|e| anyhow!("mpv render context: {e:?}"))?;
        mpv_gl.set_update_callback(move || egui_ctx.request_repaint());
        Ok(Self { mpv_gl, fbo: None, tex: None, size: (0, 0) })
    }

    /// `left_px` / `from_bottom_px` / `width_px` / `height_px` come from
    /// `PaintCallbackInfo::viewport_in_pixels()` (already converted to i32 by the caller).
    pub fn paint(&mut self, gl: &glow::Context, left_px: i32, from_bottom_px: i32, width_px: i32, height_px: i32) {
        let (w, h) = (width_px.max(1), height_px.max(1));
        unsafe {
            if self.fbo.is_none() || self.size != (w, h) {
                self.destroy(gl);
                let tex = gl.create_texture().ok();
                gl.bind_texture(glow::TEXTURE_2D, tex);
                gl.tex_image_2d(
                    glow::TEXTURE_2D, 0, glow::RGBA8 as i32, w, h, 0,
                    glow::RGBA, glow::UNSIGNED_BYTE, None,
                );
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                gl.bind_texture(glow::TEXTURE_2D, None);
                let fbo = gl.create_framebuffer().ok();
                gl.bind_framebuffer(glow::FRAMEBUFFER, fbo);
                gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, tex, 0);
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                self.tex = tex;
                self.fbo = fbo;
                self.size = (w, h);
            }
            let Some(fbo) = self.fbo else { return };
            let _ = self.mpv_gl.render::<ProcLoader>(fbo.0.get() as i32, w, h, true);

            gl.disable(glow::SCISSOR_TEST);
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fbo));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            gl.blit_framebuffer(
                0, 0, w, h,
                left_px, from_bottom_px, left_px + w, from_bottom_px + h,
                glow::COLOR_BUFFER_BIT, glow::NEAREST,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.enable(glow::SCISSOR_TEST);
        }
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            if let Some(f) = self.fbo.take() {
                gl.delete_framebuffer(f);
            }
            if let Some(t) = self.tex.take() {
                gl.delete_texture(t);
            }
        }
    }
}
