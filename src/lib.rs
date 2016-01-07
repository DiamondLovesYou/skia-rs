/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate euclid;
extern crate gleam;
extern crate libc;

#[cfg(target_os="macos")]
extern crate cgl;
#[cfg(target_os="macos")]
extern crate io_surface;

#[cfg(target_os="linux")]
extern crate x11;
#[cfg(target_os="linux")]
extern crate glx;

#[cfg(target_os="android")]
extern crate egl;

extern crate skia_sys as sys;
extern crate glium;

use std::ptr;

pub use sys::{ColorType, AlphaType, ColorProfile, TypefaceStyle,
              Align, PaintStyle,};

mod skia {
    pub use sys::*;
}

pub mod gr;

pub mod gl_context;
pub mod gl_rasterization_context;

#[cfg(target_os="linux")]
pub mod gl_context_glx;
#[cfg(target_os="linux")]
pub mod gl_rasterization_context_glx;

#[cfg(target_os="macos")]
pub mod gl_context_cgl;
#[cfg(target_os="macos")]
pub mod gl_rasterization_context_cgl;

#[cfg(target_os="android")]
pub mod gl_context_android;
#[cfg(target_os="android")]
pub mod gl_rasterization_context_android;

#[cfg(target_os="windows")]
pub mod gl_context_wgl;
#[cfg(target_os="windows")]
pub mod gl_rasterization_context_wgl;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ImageInfo {
    pub size: euclid::Size2D<i32>,
    pub color_type: ColorType,
    pub alpha_type: AlphaType,
    pub color_profile: ColorProfile,
}
impl Into<sys::ImageInfo> for ImageInfo {
    fn into(self) -> sys::ImageInfo {
        sys::ImageInfo {
            width: self.size.width,
            height: self.size.height,
            color_type: self.color_type,
            alpha_type: self.alpha_type,
            color_profile: self.color_profile,
        }
    }
}
impl From<sys::ImageInfo> for ImageInfo {
    fn from(f: sys::ImageInfo) -> ImageInfo {
        ImageInfo {
            size: euclid::Size2D::new(f.width, f.height),
            color_type: f.color_type,
            alpha_type: f.alpha_type,
            color_profile: f.color_profile,
        }
    }
}
impl Default for ImageInfo {
    fn default() -> ImageInfo {
        let sys: sys::ImageInfo = Default::default();
        From::from(sys)
    }
}

pub type FPoint = euclid::point::Point2D<f32>;
pub type FRect = euclid::SideOffsets2D<f32>;
pub type ISize = euclid::Size2D<i32>;

fn to_ffi_point(p: FPoint) -> sys::Point {
    sys::Point {
        x: p.x,
        y: p.y,
    }
}
fn from_ffi_point(p: sys::Point) -> FPoint {
    euclid::point::Point2D {
        x: p.x,
        y: p.y,
    }
}
fn to_ffi_frect(r: FRect) -> sys::Rect {
    sys::Rect {
        top: r.top,
        bottom: r.bottom,
        left: r.left,
        right: r.right,
    }
}
fn from_ffi_frect(r: sys::Rect) -> FRect {
    FRect {
        top: r.top,
        bottom: r.bottom,
        left: r.left,
        right: r.right,
    }
}
#[allow(dead_code)]
fn to_ffi_isize(p: ISize) -> sys::ISize {
    sys::ISize {
        width: p.width,
        height: p.height,
    }
}
fn from_ffi_isize(p: sys::ISize) -> ISize {
    euclid::Size2D {
        width: p.width,
        height: p.height,
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    Unknown,
    ColorType,
}

/// Aka `SkSurface`/`SkCanvas`
pub struct BasicSurface(sys::Surface);
impl Drop for BasicSurface {
    fn drop(&mut self) {
        unsafe {
            sys::sk_surface_unref(self.0);
        }
    }
}
impl Surface for BasicSurface {
    fn basic_surface(&self) -> &BasicSurface { self }
}

pub struct RasterizedSurface {
    surface: BasicSurface,
    dest: Vec<(u8, u8, u8, u8)>,
}
impl Surface for RasterizedSurface {
    fn basic_surface(&self) -> &BasicSurface { self.surface.basic_surface() }
}
impl RasterizedSurface {
    pub fn new(info: ImageInfo) -> Result<RasterizedSurface, Error> {
        if info.color_type != Default::default() {
            return Err(Error::ColorType);
        }

        let mut dest: Vec<(u8, u8, u8, u8)> = Vec::with_capacity((info.size.width * info.size.height) as usize);
        dest.resize((info.size.width * info.size.height) as usize,
                    Default::default());

        let row_size = info.size.width as usize * std::mem::size_of::<(u8, u8, u8, u8)>();
        let ptr = unsafe {
            sys::sk_new_raster_direct_surface(info.into(), dest.as_mut_ptr() as *mut _, row_size)
        };
        if ptr == ptr::null_mut() {
            Err(Error::Unknown)
        } else {
            Ok(RasterizedSurface {
                surface: BasicSurface(ptr),
                dest: dest,
            })
        }
    }

    pub fn unwrap(self) -> Vec<(u8, u8, u8, u8)> {
        let RasterizedSurface { dest, mut surface } = self;
        surface.flush();
        dest
    }
}

pub struct Paint(sys::Paint);
pub struct Path(sys::Path);
pub struct Image(sys::Image);
pub struct Typeface(sys::Typeface);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Color(sys::Color);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PathFillType {
    Winding { inverse: bool },
    EvenOdd { inverse: bool },
}
impl PathFillType {
    pub fn winding() -> PathFillType { PathFillType::Winding { inverse: false, } }
    pub fn inverse_winding() -> PathFillType { PathFillType::Winding { inverse: true, } }
    pub fn even_odd() -> PathFillType { PathFillType::EvenOdd { inverse: false } }
    pub fn inverse_even_odd() -> PathFillType { PathFillType::EvenOdd { inverse: true } }
}
impl Into<sys::PathFillType> for PathFillType {
    fn into(self) -> sys::PathFillType {
        match self {
            PathFillType::Winding { inverse: false } => sys::PathFillType::Winding,
            PathFillType::Winding { inverse: true } => sys::PathFillType::InverseWinding,
            PathFillType::EvenOdd { inverse: false } => sys::PathFillType::EvenOdd,
            PathFillType::EvenOdd { inverse: true } => sys::PathFillType::InverseEvenOdd,
        }
    }
}
impl From<sys::PathFillType> for PathFillType {
    fn from(f: sys::PathFillType) -> PathFillType {
        match f {
            sys::PathFillType::Winding => PathFillType::Winding { inverse: false },
            sys::PathFillType::InverseWinding => PathFillType::Winding { inverse: true },
            sys::PathFillType::EvenOdd => PathFillType::EvenOdd { inverse: false },
            sys::PathFillType::InverseEvenOdd => PathFillType::EvenOdd { inverse: true },
        }
    }
}

pub trait Surface {
    /// Note to implementers: it is assumed that the returned object's inner ptr is not null.
    fn basic_surface(&self) -> &BasicSurface;
}

pub trait CanvasSave: Sized + Surface {
    fn save<'s>(&'s mut self) -> Save<'s, Self>;
    fn save_layer_alpha<'a>(&'a mut self, bounds: Option<FRect>, alpha: Option<u8>) -> Save<'a, Self>;
}
impl<'a, T> CanvasSave for T
    where T: Surface,
{
    fn save<'s>(&'s mut self) -> Save<'s, T> {
        let mut count: libc::c_int = 0;
        unsafe {
            sys::sk_save(self.basic_surface().0, &mut count as *mut _);
        }

        Save {
            canvas: self,
        }
    }
    fn save_layer_alpha<'s>(&'s mut self, bounds: Option<FRect>, alpha: Option<u8>) -> Save<'s, T> {
        let bounds = bounds.map(|b| to_ffi_frect(b) );
        let bounds_ptr = bounds.as_ref()
            .map(|b| b as *const sys::Rect )
            .unwrap_or(std::ptr::null());
        unsafe {
            sys::sk_surface_save_layer_alpha(self.basic_surface().0, bounds_ptr, alpha.unwrap_or(255))
        };

        Save {
            canvas: self,
        }
    }
}

pub struct Save<'canvas, T>
    where T: Surface + 'canvas,
{
    canvas: &'canvas mut T,
}
impl<'canvas, T> Drop for Save<'canvas, T>
    where T: Surface,
{
    fn drop(&mut self) {
        unsafe {
            sys::sk_restore(self.canvas.basic_surface().0);
        }
    }
}
impl<'canvas, T> Surface for Save<'canvas, T>
    where T: Surface,
{
    fn basic_surface(&self) -> &BasicSurface {
        self.canvas.basic_surface()
    }
}

pub trait Canvas: Sized {
    fn new_image_snapshot(&self) -> Image;
    fn image_info(&self) -> ImageInfo;
    fn discard(&mut self) -> &mut Self;
    fn flush(&mut self) -> &mut Self;
    fn translate(&mut self, p: FPoint) -> &mut Self;
    fn scale(&mut self, p: FPoint) -> &mut Self;
    fn rotate(&mut self, degrees: f32) -> &mut Self;
    fn clip_rect(&mut self, rect: FRect) -> &mut Self;
    fn draw_paint(&mut self, p: &Paint) -> &mut Self;
    fn draw_line(&mut self, paint: &Paint, start: FPoint, end: FPoint) -> &mut Self;
    fn draw_points(&mut self, paint: &Paint, mode: sys::PointMode,
                   points: &[FPoint]) -> &mut Self;
    fn draw_path(&mut self, paint: &Paint, path: &Path) -> &mut Self;
    fn draw_text(&mut self, paint: &Paint, pos: FPoint, text: &str) -> &mut Self;
}

impl<'a, T> Canvas for T
    where T: Surface,
{
    fn new_image_snapshot(&self) -> Image {
        Image(unsafe {
            sys::sk_new_image_snapshot(self.basic_surface().0)
        })
    }
    fn image_info(&self) -> ImageInfo {
        From::from(unsafe {
            sys::sk_surface_get_image_info(self.basic_surface().0)
        })
    }
    fn discard(&mut self) -> &mut Self {
        unsafe {
            sys::sk_surface_discard(self.basic_surface().0);
        }
        self
    }
    fn flush(&mut self) -> &mut Self {
        unsafe {
            sys::sk_flush(self.basic_surface().0);
        }
        self
    }

    fn translate(&mut self, p: FPoint) -> &mut Self {
        unsafe {
            sys::sk_translate(self.basic_surface().0, p.x, p.y);
        }
        self
    }
    fn scale(&mut self, p: FPoint) -> &mut Self {
        unsafe {
            sys::sk_scale(self.basic_surface().0, p.x, p.y);
        }
        self
    }
    fn rotate(&mut self, degrees: f32) -> &mut Self {
        unsafe {
            sys::sk_rotate(self.basic_surface().0, degrees);
        }
        self
    }
    fn clip_rect(&mut self, rect: FRect) -> &mut Self {
        unsafe {
            sys::sk_clip_rect(self.basic_surface().0, to_ffi_frect(rect));
        }
        self
    }
    fn draw_paint(&mut self, p: &Paint) -> &mut Self {
        unsafe {
            sys::sk_draw_paint(self.basic_surface().0, p.0);
        }
        self
    }
    fn draw_line(&mut self, paint: &Paint, start: FPoint, end: FPoint) -> &mut Self {
        unsafe {
            sys::sk_surface_draw_line(self.basic_surface().0, paint.0,
                                      to_ffi_point(start),
                                      to_ffi_point(end));
        }
        self
    }
    fn draw_points(&mut self, paint: &Paint, mode: sys::PointMode,
                       points: &[FPoint]) -> &mut Self {
        unsafe {
            sys::sk_draw_points(self.basic_surface().0, paint.0, mode, points.as_ptr() as *const _,
                                points.len() as libc::size_t);
        }
        self
    }
    fn draw_path(&mut self, paint: &Paint, path: &Path) -> &mut Self {
        unsafe {
            sys::sk_draw_path(self.basic_surface().0, paint.0, path.0);
        }
        self
    }
    fn draw_text(&mut self, paint: &Paint, pos: FPoint, text: &str) -> &mut Self {
        paint.set_text_encoding(sys::TextEncoding::Utf8);
        unsafe {
            sys::sk_draw_text(self.basic_surface().0, paint.0, to_ffi_point(pos),
                              text.as_ptr() as *const _, text.len());
        }
        self
    }
}

impl Default for Paint {
    fn default() -> Paint {
        Paint(unsafe {
            sys::sk_new_paint()
        })
    }
}
impl Drop for Paint {
    fn drop(&mut self) {
        unsafe {
            sys::sk_paint_unref(self.0)
        };
    }
}
impl Clone for Paint {
    fn clone(&self) -> Paint {
        Paint(unsafe {
            sys::sk_new_paint_copy(self.0)
        })
    }
}

impl Paint {
    pub fn reset(&mut self) {
        unsafe {
            sys::sk_paint_reset(self.0)
        };
    }
    pub fn get_color(&self) -> Color {
        From::from(unsafe {
            sys::sk_paint_get_color(self.0)
        })
    }
    pub fn set_color(&mut self, color: Color) {
        unsafe {
            sys::sk_paint_set_color(self.0, color.into())
        };
    }
    pub fn get_typeface(&self) -> Option<Typeface> {
        let tf = unsafe {
            sys::sk_paint_get_typeface(self.0)
        };
        if tf == ptr::null_mut() {
            None
        } else {
            Some(Typeface(tf))
        }
    }
    pub fn set_typeface(&mut self, tf: Option<&Typeface>) {
        let tf_ptr = tf.map(|t| t.0 ).unwrap_or(ptr::null_mut());
        unsafe {
            sys::sk_paint_set_typeface(self.0, tf_ptr);
        }
    }
    pub fn get_anti_alias(&self) -> bool {
        unsafe {
            sys::sk_paint_get_anti_alias(self.0)
        }
    }
    pub fn set_anti_alias(&mut self, v: bool) {
        unsafe {
            sys::sk_paint_set_anti_alias(self.0, v);
        }
    }
    pub fn get_subpixel_text(&self) -> bool {
        unsafe {
            sys::sk_paint_get_subpixel_text(self.0)
        }
    }
    pub fn set_subpixel_text(&mut self, v: bool) {
        unsafe {
            sys::sk_paint_set_subpixel_text(self.0, v)
        }
    }
    pub fn get_lcd_render_text(&self) -> bool {
        unsafe {
            sys::sk_paint_get_lcd_render_text(self.0)
        }
    }
    pub fn set_lcd_render_text(&mut self, v: bool) {
        unsafe {
            sys::sk_paint_set_lcd_render_text(self.0, v)
        }
    }
    pub fn get_text_size(&self) -> f32 {
        unsafe {
            sys::sk_paint_get_text_size(self.0)
        }
    }
    pub fn set_text_size(&mut self, size: f32) {
        unsafe {
            sys::sk_paint_set_text_size(self.0, size)
        }
    }
    pub fn get_text_x_scale(&self) -> f32 {
        unsafe {
            sys::sk_paint_get_text_x_scale(self.0)
        }
    }
    pub fn set_text_x_scale(&mut self, p: f32) {
        unsafe {
            sys::sk_paint_set_text_x_scale(self.0, p);
        }
    }
    pub fn get_text_align(&self) -> Align {
        unsafe {
            sys::sk_paint_get_text_align(self.0)
        }
    }
    pub fn set_text_align(&mut self, a: Align) {
        unsafe {
            sys::sk_paint_set_text_align(self.0, a);
        }
    }
    pub fn get_style(&self) -> PaintStyle {
        unsafe {
            sys::sk_paint_get_style(self.0)
        }
    }
    pub fn set_style(&mut self, s: PaintStyle) {
        unsafe {
            sys::sk_paint_set_style(self.0, s);
        }
    }
    fn set_text_encoding(&self, e: sys::TextEncoding) {
        unsafe {
            sys::sk_paint_set_text_encoding(self.0, e);
        }
    }
    pub fn measure_text<T>(&self, text: T, scale: Option<f32>, bounds: Option<&mut FRect>) -> f32
        where T: AsRef<str>,
    {
        self.set_text_encoding(sys::TextEncoding::Utf8);
        let s = scale.unwrap_or(0.0f32);
        if let Some(bounds) = bounds {
            let mut ffi_bounds: sys::Rect = Default::default();
            let r = unsafe {
                sys::sk_paint_measure_text(self.0, text.as_ref().as_ptr() as *const _,
                                           text.as_ref().len(), &mut ffi_bounds as *mut _,
                                           s as libc::c_float)
            };
            *bounds = from_ffi_frect(ffi_bounds);
            return r;
        } else {
            unsafe {
                sys::sk_paint_measure_text(self.0, text.as_ref().as_ptr() as *const _,
                                           text.as_ref().len(), ptr::null_mut(),
                                           s as libc::c_float)
            }
        }
    }
}

impl Default for Path {
    fn default() -> Path {
        Path(unsafe {
            sys::sk_new_path()
        })
    }
}
impl Clone for Path {
    fn clone(&self) -> Path {
        Path(unsafe {
            sys::sk_clone_path(self.0)
        })
    }
}
impl Drop for Path {
    fn drop(&mut self) {
        unsafe {
            sys::sk_del_path(self.0)
        }
    }
}
impl Path {
    pub fn reset(&mut self) {
        unsafe {
            sys::sk_path_reset(self.0)
        };
    }
    pub fn set_fill_type(&mut self, ft: PathFillType) -> &mut Path {
        unsafe {
            sys::sk_path_set_fill_type(self.0, ft.into())
        };
        self
    }
    pub fn get_fill_type(&self) -> PathFillType {
        From::from(unsafe {
            sys::sk_path_get_fill_type(self.0)
        })
    }
    pub fn move_to(&mut self, to: FPoint, relative: bool) -> &mut Path {
        unsafe {
            sys::sk_path_move_to(self.0, to_ffi_point(to), relative)
        };
        self
    }
    pub fn line_to(&mut self, to: FPoint, relative: bool) -> &mut Path {
        unsafe {
            sys::sk_path_line_to(self.0, to_ffi_point(to), relative)
        };
        self
    }
    pub fn quad_to(&mut self, p0: FPoint, p1: FPoint, relative: bool) -> &mut Path {
        unsafe {
            sys::sk_path_quad_to(self.0, to_ffi_point(p0),
                                 to_ffi_point(p1), relative)
        };
        self
    }
    pub fn cubic_to(&mut self, p0: FPoint, p1: FPoint, p2: FPoint, relative: bool) -> &mut Path {
        unsafe {
            sys::sk_path_cubic_to(self.0, to_ffi_point(p0),
                                  to_ffi_point(p1), to_ffi_point(p2),
                                  relative)
        };
        self
    }
    pub fn close(&mut self) -> &mut Path {
        unsafe {
            sys::sk_path_close(self.0)
        };
        self
    }
    pub fn points_len(&self) -> usize {
        unsafe {
            sys::sk_path_count_points(self.0) as usize
        }
    }
    pub fn get_point(&self, idx: usize) -> FPoint {
        from_ffi_point(unsafe {
            sys::sk_path_get_point(self.0, idx as libc::c_int)
        })
    }
    pub fn image_info(&self) -> ImageInfo {
        From::from(unsafe {
            sys::sk_surface_get_image_info(self.0)
        })
    }
}

impl Into<sys::Color> for Color {
    fn into(self) -> sys::Color {
        self.0
    }
}
impl From<sys::Color> for Color {
    fn from(f: sys::Color) -> Color {
        Color(f)
    }
}
impl Color {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color(unsafe {
            sys::sk_color_from_argb(a, r, g, b)
        })
    }
    pub fn a(&self) -> u8 {
        unsafe {
            sys::sk_color_get_a(self.0) as u8
        }
    }
    pub fn r(&self) -> u8 {
        unsafe {
            sys::sk_color_get_r(self.0) as u8
        }
    }
    pub fn g(&self) -> u8 {
        unsafe {
            sys::sk_color_get_g(self.0) as u8
        }
    }
    pub fn b(&self) -> u8 {
        unsafe {
            sys::sk_color_get_b(self.0) as u8
        }
    }

    pub fn set_a(self, v: u8) -> Color {
        Color(unsafe {
            sys::sk_color_from_argb(v, self.r(), self.g(), self.b())
        })
    }
    pub fn set_r(self, v: u8) -> Color {
        Color(unsafe {
            sys::sk_color_from_argb(self.a(), v, self.g(), self.b())
        })
    }
    pub fn set_g(self, v: u8) -> Color {
        Color(unsafe {
            sys::sk_color_from_argb(self.a(), self.r(), v, self.b())
        })
    }
    pub fn set_b(self, v: u8) -> Color {
        Color(unsafe {
            sys::sk_color_from_argb(self.a(), self.r(), self.g(), v)
        })
    }
}

fn color_type_to_gl(ct: sys::ColorType) -> Option<glium::texture::UncompressedFloatFormat> {
    use glium::texture::UncompressedFloatFormat::*;
    match ct {
        sys::NATIVE_COLOR_TYPE => Some(U8U8U8U8),
        _ => None,
    }
}
fn image_info_dim_to_gl(info: &ImageInfo) -> glium::texture::Dimensions {
    glium::texture::Dimensions::Texture2d {
        width: info.size.width as u32,
        height: info.size.height as u32,
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            sys::sk_image_unref(self.0);
        }
    }
}
impl Image {
    pub fn size(&self) -> euclid::Size2D<i32> {
        let mut size: sys::ISize = unsafe { std::mem::uninitialized() };
        unsafe {
            sys::sk_image_get_size(self.0, &mut size as *mut _);
        }
        from_ffi_isize(size)
    }
    /// The returned texture object is only valid as long as the image is alive.
    pub unsafe fn get_backing_texture_handle<F>(&self, f: &F, info: ImageInfo) -> Option<glium::Texture2d>
        where F: glium::backend::Facade,
    {
        use glium::texture::texture2d::Texture2d;
        let handle = sys::sk_image_get_gr_backing_handle(self.0);
        if handle == 0 {
            None
        } else {
            let dim = image_info_dim_to_gl(&info);
            let fmt = color_type_to_gl(info.color_type);
            if fmt.is_none() { return None; }
            let fmt = fmt.unwrap();
            let tex = Texture2d::from_id(f, fmt, handle as u32, false,
                                         glium::texture::MipmapsOption::AutoGeneratedMipmaps,
                                         dim);
            Some(tex)
        }
    }
}

unsafe impl Send for Typeface { }
impl Clone for Typeface {
    fn clone(&self) -> Typeface {
        unsafe {
            sys::sk_typeface_ref(self.0);
        }
        Typeface(self.0)
    }
}
impl Drop for Typeface {
    fn drop(&mut self) {
        unsafe {
            sys::sk_typeface_unref(self.0);
        }
    }
}
impl Default for Typeface {
    fn default() -> Typeface {
        Typeface::new_from_typeface(None, Default::default())
    }
}
impl Typeface {
    pub fn new_from_name(name: &str, style: TypefaceStyle) -> Typeface {
        Typeface(unsafe {
            sys::sk_typeface_create_from_name(name.as_ptr() as *const _, name.len(),
                                              style)
        })
    }
    pub fn new_from_typeface(tf: Option<&Typeface>, style: TypefaceStyle) -> Typeface {
        Typeface(unsafe {
            let tf = tf.map(|t| t.0 ).unwrap_or(ptr::null_mut());
            sys::sk_typeface_create_from_typeface(tf, style)
        })
    }
    pub fn new_from_path<T>(path: T) -> Option<Typeface>
        where T: AsRef<std::path::Path>,
    {
        let p_str = path.as_ref().to_str();
        if p_str.is_none() { return None; }
        let p_str = p_str.unwrap();

        let tf = unsafe {
            sys::sk_typeface_create_from_path(p_str.as_ptr() as *const _, p_str.len())
        };
        if tf == ptr::null_mut() {
            None
        } else {
            Some(Typeface(tf))
        }
    }
}
