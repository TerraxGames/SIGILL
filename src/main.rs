use client::{rendering::RenderData, ClientData};
use environment::Side;
use hecs::World;
use winit::{event::WindowEvent, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowAttributes}};

pub use ::log::{error, warn, info, debug, trace}; // easy logging anywhere

mod log;
mod constants;
mod event;
mod environment;
mod client;
mod util;

struct App {
    side: Side,
    client_data: Option<ClientData>,
    world: World,
}

impl App {
    pub fn new_client(attributes: winit::window::WindowAttributes) -> Self {
        Self::new(
            Side::Client,
            Some(ClientData { window: None, attributes, render_data: None })
        )
    }

    pub fn new(side: Side, client_data: Option<ClientData>) -> Self {
        Self {
            side,
            client_data,
            world: World::new(),
        }
    }

    pub const fn client_data(&self) -> Option<&ClientData> {
        self.client_data.as_ref()
    }
    
    fn client_data_mut(&mut self) -> &mut ClientData {
        client_only!(self.side, {
            self.client_data.as_mut().unwrap()
        })
    }

    pub fn attributes(&self) -> winit::window::WindowAttributes {
        client_only!(self.side, {
            self.client_data().unwrap().attributes.clone()
        })
    }

    pub fn window(&self) -> &Window {
        client_only!(self.side, {
            self.client_data().unwrap().window.as_ref().expect("the window should be initialized before being accessed")
        })
    }

    pub fn render_data(&self) -> &RenderData {
        client_only!(self.side, {
            self.client_data().unwrap().render_data.as_ref().expect("rendering should be initialized before accessing rendering data")
        })
    }

    fn render_data_mut(&mut self) -> &mut RenderData {
        client_only!(self.side, {
            self.client_data_mut().render_data.as_mut().expect("rendering should be initialized before accessing rendering data")
        })
    }

    pub fn side(&self) -> Side {
        self.side
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let init_renderer = self.client_data().unwrap().window.is_none();
        self.client_data_mut().window = Some(event_loop.create_window(self.attributes()).unwrap());
        if init_renderer {
            client::rendering::init(self, event_loop).expect("failed to initialize rendering")
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed => {
                // Drop rendering data
                let mut none = None;
                core::mem::swap(&mut self.client_data_mut().render_data, &mut none);
                drop(none);
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                client::rendering::render(self).expect("error redrawing");
            },
            _ => (),
        }
    }
}

fn main() {
    // Initialize logging
    log::init().expect("logger initialization failed");
    log::hook_panic();

    // Initialize event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Initialize window
    let window_attributes = WindowAttributes::default()
        .with_title(constants::NAME);
    let mut app = App::new_client(window_attributes);

    info!("Initializing with side `{}`", app.side());

    // Start event loop
    event_loop.run_app(&mut app).unwrap();
}
