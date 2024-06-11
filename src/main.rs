use anyhow::Result;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod base;
use base::app::App;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Vulkan Testing")
        .with_inner_size(LogicalSize::new(800, 800))
        .build(&event_loop)?;

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, elwt| {
        match event {
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested if !elwt.exiting() => unsafe { app.render(&window) }.unwrap(),
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    unsafe { app.destroy(); }
                },
                _ => {}
            },
            _ => {}
        }
    })?;

    Ok(())
}
