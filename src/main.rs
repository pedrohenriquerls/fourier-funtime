use raylib::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Copy)]
struct Complex {
    re: f32,
    im: f32,
}

impl Complex {
    fn new(re: f32, im: f32) -> Self {
        Complex { re, im }
    }

    fn magnitude(&self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    fn phase(&self) -> f32 {
        self.im.atan2(self.re)
    }

    fn rotate(&self, angle: f32) -> Complex {
        let cos = angle.cos();
        let sin = angle.sin();
        Complex {
            re: self.re * cos - self.im * sin,
            im: self.re * sin + self.im * cos,
        }
    }

    fn add(&self, other: &Complex) -> Complex {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

struct FourierComponent {
    freq: i32,
    coef: Complex,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Fourier Series Visualization")
        .build();

    let center = Vector2::new(400.0, 300.0);
    let mut time: f32 = 0.0;
    let dt: f32 = 0.0002;

    let components: Vec<FourierComponent> = vec![
        (-3, Complex::new(30.0, 0.0)),
        (-2, Complex::new(20.0, 0.0)),
        (-1, Complex::new(10.0, 0.0)),
        (-1, Complex::new(10.0, 0.0)),
        (-1, Complex::new(10.0, 0.0)),
        (0, Complex::new(0.0, 0.0)),
        (1, Complex::new(1.0, 0.0)),
        (2, Complex::new(2.0, 0.0)),
        (3, Complex::new(3.0, 0.0)),
    ]
    .into_iter()
    .map(|(freq, coef)| FourierComponent { freq, coef })
    .collect();

    let mut path: Vec<Vector2> = vec![];

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        let mut current = Complex::new(0.0, 0.0);

        for comp in &components {
            let prev = current;

            let angle = 2.0 * PI * comp.freq as f32 * time;
            let rot = comp.coef.rotate(angle);
            current = current.add(&rot);

            let start = Vector2::new(center.x + prev.re, center.y + prev.im);
            let end = Vector2::new(center.x + current.re, center.y + current.im);

            d.draw_circle_lines(start.x as i32, start.y as i32, rot.magnitude(), Color::DARKGRAY);
            d.draw_line_ex(start, end, 2.0, Color::WHITE);
        }
        let tip = Vector2::new(center.x + current.re, center.y + current.im);
        path.insert(0, tip);
        if path.len() > 5000 {
            path.pop();
        }
        let max_alpha = 255.0;
        let len = path.len().min(5000);
        for i in 1..len {
            let alpha = ((1.0 - (i as f32 / len as f32)) * max_alpha) as u8;
            let fade_color = Color::new(0, 255, 0, alpha); // green fading
            d.draw_line_ex(path[i - 1], path[i], 2.0, fade_color);
        }

        time += dt;
    }
}
