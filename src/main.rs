#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate sdl2;

use std::{env, process, thread};
use std::time::Duration;
use std::path::PathBuf;
use std::io::{self, Write};
use sdl2::surface::Surface;
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;


mod errors {
    error_chain!{}
}

use errors::*;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

trait RendererHelpers {
    fn load_texture(&self, lesson: &str, file: &str) -> Result<Texture>;
    fn render_texture(&mut self, texture: &Texture, x: i32, y: i32) -> Result<()>;
}

impl<'a> RendererHelpers for Renderer<'a> {
    fn load_texture(&self, lesson: &str, file: &str) -> Result<Texture> {
        let path = get_resource_path(lesson, file)?;
        let surface = Surface::load_bmp(&path)?;
        self.create_texture_from_surface(surface)
            .chain_err(|| format!("Failed to create texture from surface made with {:?}", path))
    }

    fn render_texture(&mut self, texture: &Texture, x: i32, y: i32) -> Result<()> {
        let info = texture.query();
        self.copy(&texture,
                  None,
                  Some(Rect::new(x, y, info.width, info.height)))
            .map_err(From::from)
    }
}


fn main() {
    if let Err(ref e) = run() {
        let stderr = &mut io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // Set `RUST_BACKTRACE=1` to see backtrace
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        process::exit(1);
    }
}

fn run() -> Result<()> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("sdl2 tutorial", SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable()
        .allow_highdpi()
        .position_centered()
        .build()
        .chain_err(|| "Failed to build main window")?;

    println!("drawable size {:?}", window.drawable_size());
    println!("display dpi {:?}", video_subsystem.display_dpi(0));

    let mut renderer = window.renderer()
        .present_vsync()
        .build()
        .chain_err(|| "Failed to build sdl renderer")?;

    println!("output size {:?}", renderer.output_size()?);

    let background = renderer.load_texture("lesson2", "background.bmp")?;
    let image = renderer.load_texture("lesson2", "image.bmp")?;

    renderer.clear();

    let bg_info = background.query();
    renderer.render_texture(&background, 0, 0)?;
    renderer.render_texture(&background, bg_info.width as i32, 0)?;
    renderer.render_texture(&background, 0, bg_info.height as i32)?;
    renderer.render_texture(&background, bg_info.width as i32, bg_info.height as i32)?;

    let image_info = image.query();
    let image_x = (SCREEN_WIDTH - image_info.width) / 2;
    let image_y = (SCREEN_HEIGHT - image_info.height) / 2;
    renderer.render_texture(&image, image_x as i32, image_y as i32)?;

    renderer.present();
    thread::sleep(Duration::from_secs(3));

    Ok(())
}

fn get_resource_path(lesson: &str, file: &str) -> Result<PathBuf> {
    let cwd = env::current_dir().chain_err(|| "Couldn't get current directory")?;
    Ok(cwd.join("res").join(lesson).join(file))
}
