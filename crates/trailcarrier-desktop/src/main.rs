use std::{
    fs,
    path::{Path, PathBuf},
};

use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};
use trailcarrier_core::Status;
use trailcarrier_render::View;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let mut view = View::preview();
    let mut capture = |view: &View, name: &str, dark| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(view, &mut canvas, dark),
        )
    };

    capture(&view, "trailcarrier.png", false)?;
    assert_eq!(view.input(Input::Tap(900.0, 1363.0)), Change::Partial);
    assert_eq!(view.game.moves_left(), 9);
    for (x, y) in [(3, 2), (3, 3), (3, 4), (4, 4)] {
        assert_eq!(
            view.input(Input::Tap(
                212.0 + x as f32 * 120.0,
                395.0 + y as f32 * 120.0
            )),
            Change::Partial
        );
    }
    assert_eq!(view.game.parcels_left(), 0);
    capture(&view, "trailcarrier-playing-dark.png", true)?;
    for (x, y) in [(5, 4), (5, 5), (5, 6), (6, 6)] {
        assert_eq!(
            view.input(Input::Tap(
                212.0 + x as f32 * 120.0,
                395.0 + y as f32 * 120.0
            )),
            Change::Partial
        );
    }
    assert_eq!(view.game.status(), Status::Complete);
    capture(&view, "trailcarrier-complete.png", false)?;
    assert_eq!(view.input(Input::Tap(920.0, 1586.0)), Change::Full);
    assert_eq!(view.game.level(), 1);
    assert_eq!(view.input(Input::Tap(1110.0, 90.0)), Change::Exit);
    Ok(())
}
