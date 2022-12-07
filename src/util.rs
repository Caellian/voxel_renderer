use std::borrow::Cow;

#[cfg(target_arch = "wasm32")]
pub fn insert_canvas() {
    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    window.set_inner_size(PhysicalSize::new(450, 400));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("earth-!SECTIONoxide")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

pub type CowStr<'a> = Cow<'a, str>;

pub trait UninitVec {
    unsafe fn new_uninit(capacity: usize) -> Self;
}

impl<T> UninitVec for Vec<T> {
    #[inline(always)]
    unsafe fn new_uninit(capacity: usize) -> Self {
        let result = Vec::with_capacity(capacity);
        result.set_len(capacity);
        result
    }
}
