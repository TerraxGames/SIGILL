use rendering::RenderData;

pub mod rendering;

pub struct ClientData {
    pub window: Option<winit::window::Window>,
    pub attributes: winit::window::WindowAttributes,
    pub render_data: Option<RenderData>,
}
