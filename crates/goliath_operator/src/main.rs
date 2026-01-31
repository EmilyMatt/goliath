mod error;

use sdl2::rect::Rect;
use crate::error::{GoliathOperatorError, GoliathOperatorResult};
use goliath_common::common_init_for_trace;

enum ApplicationState {
    Connect
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> GoliathOperatorResult<()> {
    common_init_for_trace()?;

    let sdl_context = sdl2::init().map_err(GoliathOperatorError::SdlError)?;
    let video_subsystem = sdl_context.video().map_err(GoliathOperatorError::SdlError)?;
    let joystick_subsystem = sdl_context.joystick().map_err(GoliathOperatorError::SdlError)?;

    let available = joystick_subsystem
        .num_joysticks().map_err(GoliathOperatorError::SdlError)?;


    log::info!("Number of joysticks: {}", available);

    let bounds = video_subsystem.display_bounds(0).map_err(GoliathOperatorError::SdlError)?;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", bounds.width(), bounds.height())
        .fullscreen()
        .opengl()
        .build()?;
    let mut canvas = window.into_canvas().build()?;

    let application_state = ApplicationState::Connect;
    loop {
        match application_state {
            ApplicationState::Connect => {
                canvas.draw_rect(Rect::new(bounds.width() as i32 / 2 - 100, bounds.height() as i32 / 2 - 100, 200, 200)).map_err(GoliathOperatorError::SdlError)?;
            }
        }
    }
}
