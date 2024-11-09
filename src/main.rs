use std::sync::Arc;

use anyhow::Result;
use application::Application;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    window::Window,
};

mod application;

fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = ApplicationWindow::new(&event_loop);
    event_loop.run_app(&mut app)?;

    Ok(())
}

pub enum UserEvent {
    ApplicationCreated(Application),
}

pub struct ApplicationWindow {
    app: Option<Application>,
    window: Option<Arc<Window>>,
    close_requested: bool,
    event_proxy: EventLoopProxy<UserEvent>,
}

impl ApplicationWindow {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        Self {
            window: None,
            app: None,
            close_requested: false,
            event_proxy: event_loop.create_proxy(),
        }
    }
}

async fn create_application(window: Arc<Window>, size: LogicalSize<u32>, event_proxy: EventLoopProxy<UserEvent>) {
    let size = size.to_physical(window.scale_factor());
    log::info!("Initial size: {}x{}", size.width, size.height);
    let app = Application::new(window, size)
        .await
        .expect("creation of application failed");
    event_proxy
        .send_event(UserEvent::ApplicationCreated(app))
        .map_err(|_| "sending created application failed")
        .unwrap();
}

impl ApplicationHandler<UserEvent> for ApplicationWindow {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = LogicalSize::new(1280, 720);
        let window_attributes = Window::default_attributes()
            .with_title("wgpu raytracer")
            .with_inner_size(size)
            .with_min_inner_size(LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap()); // needed for resize closure on web
        self.window = Some(window.clone());

        #[cfg(target_arch = "wasm32")]
        let size = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            // On wasm, append the canvas to the document body
            let window = window.clone();
            let canvas = window.canvas().expect("couldn't retrieve canvas");
            let web_window = web_sys::window().expect("couldn't retrieve website window");
            let body = web_window
                .document()
                .and_then(|doc| doc.body())
                .expect("couldn't retrieve document body");
            body.append_child(&web_sys::Element::from(canvas))
                .ok()
                .expect("couldn't append canvas to body");

            let window_size = LogicalSize::new(
                web_window.inner_width().unwrap().as_f64().unwrap() as u32,
                web_window.inner_height().unwrap().as_f64().unwrap() as u32,
            );
            let _ = window.request_inner_size(window_size);

            let resize_closure =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                    let web_window = web_sys::window().expect("couldn't retrieve website window");
                    log::info!("bres {} {}", body.client_width(), body.client_height());
                    let _ = window.request_inner_size(LogicalSize::new(
                        web_window.inner_width().unwrap().as_f64().unwrap(),
                        web_window.inner_height().unwrap().as_f64().unwrap(),
                    ));
                }) as Box<dyn FnMut(_)>);
            web_window
                .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
                .unwrap();
            resize_closure.forget();

            window_size
        };

        let event_proxy = self.event_proxy.clone();
        #[cfg(not(target_arch = "wasm32"))]
        futures::executor::block_on(create_application(window, size, event_proxy));
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(create_application(window, size, event_proxy));
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ApplicationCreated(application) => {
                self.app = Some(application);
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(app), Some(window)) = (&mut self.app, &self.window) else {
            return;
        };
        if app.handle_event(&window, &event) {
            return;
        }

        match event {
            WindowEvent::Resized(size) => app.resize(&window, size.width, size.height),
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = app.render(&window) {
                    if e == wgpu::SurfaceError::Outdated {
                        let size = window.inner_size();
                        app.resize(&window, size.width, size.height);
                    } else {
                        panic!("{}", e);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.close_requested {
            event_loop.exit();
            return;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
