use web_sys::HtmlCanvasElement;

use nalgebra::Vector2;

pub fn device_pixel_ratio() -> f64 {
    let window = web_sys::window().expect("Failed to obtain window");
    window.device_pixel_ratio()
}

pub fn set_canvas_size(canvas: &HtmlCanvasElement, logical_size: Vector2<u32>) {
    let scale_factor = device_pixel_ratio();

    canvas.set_width((logical_size.x as f64 * scale_factor).round() as u32);
    canvas.set_height((logical_size.y as f64 * scale_factor).round() as u32);

    set_canvas_style_property(canvas, "width", &format!("{}px", logical_size.x));
    set_canvas_style_property(canvas, "height", &format!("{}px", logical_size.y));
}

pub fn set_canvas_style_property(canvas: &HtmlCanvasElement, property: &str, value: &str) {
    let style = canvas.style();
    style
        .set_property(property, value)
        .unwrap_or_else(|_| panic!("Failed to set {}", property));
}
