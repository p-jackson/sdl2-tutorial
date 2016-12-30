#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate sdl2;

use std::{env, process};
use std::path::PathBuf;
use std::io::{self, Write};
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
use sdl2::image;
use sdl2::ttf::{self, Font};
use sdl2::pixels::Color;


mod errors {
    error_chain!{}
}

use errors::*;


const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;
const TILE_SIZE: u32 = 40;


trait RendererHelpers {
    fn load_texture_resource(&self, lesson: &str, file: &str) -> Result<Texture>;
    fn render_texture(&mut self, texture: &Texture, x: i32, y: i32) -> Result<()>;
    fn render_texture_with_size(&mut self,
                                texture: &Texture,
                                x: i32,
                                y: i32,
                                w: u32,
                                h: u32)
                                -> Result<()>;
    fn render_text(&mut self, text: &str, font: &Font, color: Color) -> Result<Texture>;
}


impl<'a> RendererHelpers for Renderer<'a> {
    fn load_texture_resource(&self, lesson: &str, file: &str) -> Result<Texture> {
        use sdl2::image::LoadTexture;
        let path = get_resource_path(lesson, file)?;
        self.load_texture(&path).map_err(From::from)
    }

    fn render_texture(&mut self, texture: &Texture, x: i32, y: i32) -> Result<()> {
        let info = texture.query();
        self.render_texture_with_size(texture, x, y, info.width, info.height)
    }

    fn render_texture_with_size(&mut self,
                                texture: &Texture,
                                x: i32,
                                y: i32,
                                w: u32,
                                h: u32)
                                -> Result<()> {
        self.copy(&texture, None, Some(Rect::new(x, y, w, h)))
            .map_err(From::from)
    }

    fn render_text(&mut self, text: &str, font: &Font, color: Color) -> Result<Texture> {
        let surface = font.render(text)
            .blended(color)
            .chain_err(|| format!("Failed to render text to surface: {}", text))?;
        self.create_texture_from_surface(surface)
            .chain_err(|| format!("Failed to create texture from text: {}", text))
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
    let _image_context = image::init(image::INIT_PNG)?;
    let ttf_context = ttf::init().chain_err(|| "Failed to init ttf context")?;

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

    let background = renderer.load_texture_resource("lesson3", "background.png")?;
    let font_path = get_resource_path("lesson6", "sample.ttf")?;
    let font = ttf_context.load_font(&font_path, 64)?;
    let color = Color::RGBA(255, 255, 255, 255);
    let image = renderer.render_text("TTF fonts are cool!", &font, color)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut quit = false;

    while !quit {
        while let Some(event) = event_pump.poll_event() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => quit = true,
                Event::KeyDown { .. } => quit = true,
                Event::MouseButtonDown { .. } => quit = true,
                _ => (),
            }
        }

        renderer.clear();

        let x_tiles = SCREEN_WIDTH / TILE_SIZE;
        let y_tiles = SCREEN_HEIGHT / TILE_SIZE;

        for i in 0..(x_tiles * y_tiles) {
            let x = i % x_tiles;
            let y = i / x_tiles;
            renderer.render_texture_with_size(&background,
                                          (x * TILE_SIZE) as i32,
                                          (y * TILE_SIZE) as i32,
                                          TILE_SIZE,
                                          TILE_SIZE)?;
        }

        let image_info = image.query();
        let image_x = (SCREEN_WIDTH - image_info.width) / 2;
        let image_y = (SCREEN_HEIGHT - image_info.height) / 2;
        renderer.render_texture(&image, image_x as i32, image_y as i32)?;

        renderer.present();
    }

    Ok(())
}


fn get_resource_path(lesson: &str, file: &str) -> Result<PathBuf> {
    let cwd = env::current_dir().chain_err(|| "Couldn't get current directory")?;
    Ok(cwd.join("res").join(lesson).join(file))
}
