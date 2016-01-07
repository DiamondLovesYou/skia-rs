//! Note: this doesn't work with Glium currently.

use std::ptr;

use glium;
use sys;

use super::{ImageInfo, BasicSurface};

pub struct Context(sys::SkiaGrContextRef);
pub struct GlInterface(sys::SkiaGrGLInterfaceRef);

impl Drop for GlInterface {
    fn drop(&mut self) {
        unsafe {
            sys::SkiaGrGLInterfaceRelease(self.0)
        }
    }
}
impl GlInterface {
    pub fn new_native<T>(context: &T) -> Option<GlInterface>
        where T: glium::backend::Backend,
    {
        unsafe { context.make_current() };
        let iface = unsafe { sys::SkiaGrGLCreateNativeInterface() };
        if iface == ptr::null_mut() {
            return None;
        } else {
            return Some(GlInterface(iface));
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::SkiaGrContextRelease(self.0)
        }
    }
}
impl Context {
    pub fn new_gl(iface: GlInterface) -> Option<Context> {
        let ctxt = unsafe { sys::SkiaGrContextCreate(iface.0) };
        if ctxt == ptr::null_mut() {
            None
        } else {
            Some(Context(ctxt))
        }
    }

    pub fn create_budgeted_offscreen_surface(&self,
                                             info: ImageInfo) -> Option<BasicSurface> {
        let surface = unsafe {
            sys::sk_new_render_target_surface(self.0, sys::CacheManagement::Budgeted,
                                              info.into())
        };
        if surface == ptr::null_mut() {
            None
        } else {
            Some(BasicSurface(surface))
        }
    }

    pub fn flush(&self, discard: bool) {
        let flags = if discard { sys::GrContextFlushFlags::Discard }
                    else { sys::GrContextFlushFlags::None };
        unsafe {
            sys::gr_context_flush(self.0, flags);
        }
    }
}
