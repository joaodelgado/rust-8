use sdl2::Sdl;
use sdl2::render::Renderer;
use sdl2::rect::Point;
use sdl2::pixels::Color;

use spec;

#[derive(Debug)]
pub struct Pixel {
    x: usize,
    y: usize,
    value: u8,
}

impl Pixel {
    pub fn new(x: usize, y: usize, value: u8) -> Pixel {
        Pixel {
            x: x,
            y: y,
            value: value,
        }
    }

    pub fn as_point(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }

    pub fn as_color(&self) -> Color {
        match self.value {
            0 => Color::RGB(0, 0, 0),
            _ => Color::RGB(255, 255, 255),
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}

pub struct Display<'a> {
    renderer: Renderer<'a>,
    pixels: [[u8; spec::DISPLAY_WIDTH as usize]; spec::DISPLAY_HEIGHT as usize],
}

impl<'a> Display<'a> {
    pub fn new(sdl_context: &Sdl) -> Display<'a> {
        let video_subsytem = sdl_context.video().unwrap();

        let window = video_subsytem.window(spec::WINDOW_NAME,
                                           spec::DISPLAY_WIDTH * spec::DISPLAY_SCALE,
                                           spec::DISPLAY_HEIGHT * spec::DISPLAY_SCALE)
                                   .position_centered()
                                   .opengl()
                                   .build()
                                   .unwrap();
        let mut renderer = window.renderer().build().unwrap();
        let scale = spec::DISPLAY_SCALE as f32;
        let _ = renderer.set_scale(scale, scale);

        Display {
            renderer: renderer,
            pixels: [[0u8; spec::DISPLAY_WIDTH as usize]; spec::DISPLAY_HEIGHT as usize],
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        self.pixels[y][x]
    }

    pub fn draw(&mut self, pixels: Vec<Pixel>) {
        for pixel in pixels.into_iter() {
            let point = pixel.as_point();
            self.pixels[point.y() as usize][point.x() as usize] = pixel.value();
            let _ = self.renderer.set_draw_color(pixel.as_color());
            let _ = self.renderer.draw_point(pixel.as_point());
        }
    }

    pub fn flush(&mut self) {
        let _ = self.renderer.present();
    }
}
