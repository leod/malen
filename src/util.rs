use web_sys::HtmlCanvasElement;

use nalgebra::Vector2;

pub fn device_pixel_ratio() -> f64 {
    let window = web_sys::window().expect("Failed to obtain window");
    window.device_pixel_ratio()
}

pub fn logical_to_physical_size(logical_size: Vector2<u32>) -> Vector2<u32> {
    let scale_factor = device_pixel_ratio();

    // Scale factor is often non-integer, so we need to map to integer somehow.
    // It is not defined in the HTML spec which function should be used for
    // this.
    //
    // https://webgl2fundamentals.org/webgl/lessons/webgl-resizing-the-canvas.html

    Vector2::new(
        (logical_size.x as f64 * scale_factor).round() as u32,
        (logical_size.y as f64 * scale_factor).round() as u32,
    )
}

pub fn set_canvas_physical_size(canvas: &HtmlCanvasElement, physical_size: Vector2<u32>) {
    canvas.set_width(physical_size.x);
    canvas.set_height(physical_size.y);
}

pub fn set_canvas_logical_size(canvas: &HtmlCanvasElement, logical_size: Vector2<u32>) {
    set_canvas_style_property(canvas, "width", &format!("{}px", logical_size.x));
    set_canvas_style_property(canvas, "height", &format!("{}px", logical_size.y));
}

pub fn set_canvas_logical_size_fill(canvas: &HtmlCanvasElement) {
    set_canvas_style_property(canvas, "width", "100vw");
    set_canvas_style_property(canvas, "height", "100vh");
}

pub fn set_canvas_style_property(canvas: &HtmlCanvasElement, property: &str, value: &str) {
    let style = canvas.style();
    style
        .set_property(property, value)
        .unwrap_or_else(|_| panic!("Failed to set {}", property));
}

pub fn make_canvas_focusable(canvas: &HtmlCanvasElement) {
    canvas.set_attribute("tabIndex", "1").unwrap();
}
