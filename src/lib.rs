pub use glutin;
pub use exgui;
pub use gl;

use std::mem;
use glutin::{WindowBuilder, ContextBuilder, EventsLoop, GlWindow, GlContext, ElementState,
             CreationError, ContextError, MouseButton};
use derive_more::From as FromMode;
use exgui::{Comp, Color, controller::MouseInput};

pub enum AppState {
    Exit,
    Continue,
}

pub struct App {
    events_loop: Option<EventsLoop>,
    window: GlWindow,
    background_color: Color,
    exit_by_escape: bool,
    width: u32,
    height: u32,
}

#[derive(Debug, FromMode)]
pub enum AppError {
    CreationError(CreationError),
    ContextError(ContextError),
    WindowNoLongerExists,
    EventsLoopIsNone,
}

impl App {
    pub fn new(
        window_builder: WindowBuilder,
        context_builder: ContextBuilder,
    ) -> Result<Self, AppError>
    {
        let events_loop = EventsLoop::new();
        let (width, height) = window_builder.window.max_dimensions.unwrap_or((0, 0));
        let window = GlWindow::new(window_builder, context_builder, &events_loop)?;
        Ok(App {
            events_loop: Some(events_loop),
            window,
            background_color: Color::RGBA(0.8, 0.8, 0.8, 1.0),
            width,
            height,
            exit_by_escape: true,
        })
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn with_exit_by_escape(mut self, exit: bool) -> Self {
        self.exit_by_escape = exit;
        self
    }

    pub fn init(&mut self) -> Result<&mut Self, AppError> {
        unsafe {
            self.window.make_current()?;
            gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
            let color = self.background_color.as_arr();
            gl::ClearColor(color[0], color[1], color[2], color[3]);
        }
        Ok(self)
    }

    #[inline]
    pub fn run(&mut self, comp: &mut Comp) -> Result<(), AppError> {
        self.run_proc(comp, |_, _| AppState::Continue)
    }

    pub fn run_proc(&mut self, comp: &mut Comp, mut proc: impl FnMut(&App, &mut Comp) -> AppState)
        -> Result<(), AppError>
    {
        let mut mouse_controller = MouseInput::new();
        let mut events_loop = mem::replace(&mut self.events_loop, None)
            .ok_or(AppError::EventsLoopIsNone)?;
        let mut running = true;
        while running {
            let (width, height) = self.window.get_inner_size()
                .ok_or(AppError::WindowNoLongerExists)?;
            self.width = width;
            self.height = height;
            unsafe {
                gl::Viewport(0, 0, width as i32, height as i32);
                gl::Clear(
                    gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
                );
            }

            if let AppState::Exit = proc(self, comp) {
                break;
            }

            self.window.swap_buffers()?;

            events_loop.poll_events(|event| match event {
                glutin::Event::WindowEvent { event, .. } => {
                    match event {
                        glutin::WindowEvent::Closed  => running = false,
                        glutin::WindowEvent::KeyboardInput {
                            input: glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } if self.exit_by_escape => running = false,
                        glutin::WindowEvent::Resized(w, h) => self.window.resize(w, h),
                        glutin::WindowEvent::CursorMoved { position: (x_pos, y_pos), .. } => {
                            mouse_controller.update_pos(x_pos, y_pos);
                        },
                        glutin::WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                            mouse_controller.left_pressed_comp(comp);
                        },
                        _ => (),
                    }
                }
                _ => (),
            });
        }
        mem::replace(&mut self.events_loop, Some(events_loop));

        Ok(())
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn window(&self) -> &GlWindow {
        &self.window
    }
}