// This code is based in the video https://www.youtube.com/watch?v=r6sGWTCMz2k
// magnitude: length of the vector (distance from origin).
// phase: angle of the vector in radians.
// rotate(angle): rotates the complex number by angle radians (using 2D rotation formula).
// add: adds two complex numbers component-wise.

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

    fn scale(&self, scalar: f32) -> Complex {
        Complex {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }

    fn multiply(&self, other: &Complex) -> Complex {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

struct FourierComponent {
    freq: i32,
    coef: Complex,
}

struct PathFourier {
    components: Vec<FourierComponent>,
    center: Vector2,
    color: Color,
    path: Vec<Vector2>,
}

impl PathFourier {
    fn new(path_points: &[Vector2], center: Vector2, color: Color, num_components: usize) -> Self {
        if path_points.is_empty() {
            return Self {
                components: vec![],
                center,
                color,
                path: vec![],
            };
        }

        let path_center_x = path_points.iter().map(|p| p.x).sum::<f32>() / path_points.len() as f32;
        let path_center_y = path_points.iter().map(|p| p.y).sum::<f32>() / path_points.len() as f32;
        
        let complex_signal: Vec<Complex> = path_points
            .iter()
            .map(|p| Complex::new(p.x - path_center_x, p.y - path_center_y))
            .collect();

        let components = compute_complex_dft(&complex_signal, num_components);

        Self {
            components,
            center,
            color,
            path: vec![],
        }
    }

    fn evaluate(&self, time: f32) -> Vector2 {
        let mut sum = Complex::new(0.0, 0.0);

        for comp in &self.components {
            let angle = 2.0 * PI * comp.freq as f32 * time;
            let rotated = comp.coef.rotate(angle);
            sum = sum.add(&rotated);
        }

        Vector2::new(self.center.x + sum.re, self.center.y + sum.im)
    }

    fn update_path(&mut self, point: Vector2, max_path_length: usize) {
        self.path.insert(0, point);
        if self.path.len() > max_path_length {
            self.path.pop();
        }
    }

    fn draw_vectors(&self, d: &mut RaylibDrawHandle, time: f32, show_circles: bool) {
        let mut current = Complex::new(0.0, 0.0);

        for comp in &self.components {
            let prev = current;
            let angle = 2.0 * PI * comp.freq as f32 * time;
            let rot = comp.coef.rotate(angle);
            current = current.add(&rot);

            let prev_pos = Vector2::new(
                self.center.x + prev.re,
                self.center.y + prev.im,
            );
            let current_pos = Vector2::new(
                self.center.x + current.re,
                self.center.y + current.im,
            );

            if show_circles && comp.coef.magnitude() > 0.1 {
                d.draw_circle_lines(
                    prev_pos.x as i32,
                    prev_pos.y as i32,
                    comp.coef.magnitude(),
                    Color::new(150, 150, 150, 180),
                );
            }

            d.draw_line_ex(prev_pos, current_pos, 2.5, Color::WHITE);
            d.draw_circle(current_pos.x as i32, current_pos.y as i32, 3.0, Color::WHITE);
        }
    }

    fn draw_path(&self, d: &mut RaylibDrawHandle) {
        if self.path.len() < 2 {
            return;
        }

        let max_alpha = 255.0;
        let len = self.path.len().min(2000);
        for i in 1..len {
            let alpha = ((1.0 - (i as f32 / len as f32)) * max_alpha) as u8;
            let fade_color = Color::new(
                self.color.r,
                self.color.g,
                self.color.b,
                alpha,
            );
            d.draw_line_ex(self.path[i - 1], self.path[i], 2.0, fade_color);
        }
    }
}

fn compute_complex_dft(signal: &[Complex], num_components: usize) -> Vec<FourierComponent> {
    let n = signal.len();
    if n == 0 {
        return vec![];
    }

    let mut components = Vec::new();
    
    for k in 0..n {
        let mut sum = Complex::new(0.0, 0.0);

        for (i, &value) in signal.iter().enumerate() {
            let angle = -2.0 * PI * k as f32 * i as f32 / n as f32;
            let exp_term = Complex::new(angle.cos(), angle.sin());
            let product = exp_term.multiply(&value);
            sum = sum.add(&product);
        }

        let coef = sum.scale(1.0 / n as f32);
        let freq = if k <= n / 2 {
            k as i32
        } else {
            k as i32 - n as i32
        };
        
        components.push(FourierComponent { freq, coef });
    }

    components.sort_by(|a, b| {
        b.coef.magnitude().partial_cmp(&a.coef.magnitude()).unwrap()
    });

    components.truncate(num_components);
    components.sort_by_key(|c| c.freq);
    components
}

fn create_square_path(size: f32) -> Vec<Vector2> {
    let half = size / 2.0;
    let mut points = Vec::new();
    let steps = 200;

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let point = if t < 0.25 {
            Vector2::new(-half + t * 4.0 * size, -half)
        } else if t < 0.5 {
            Vector2::new(half, -half + (t - 0.25) * 4.0 * size)
        } else if t < 0.75 {
            Vector2::new(half - (t - 0.5) * 4.0 * size, half)
        } else {
            Vector2::new(-half, half - (t - 0.75) * 4.0 * size)
        };
        points.push(point);
    }

    points
}

fn create_circle_path(radius: f32) -> Vec<Vector2> {
    let mut points = Vec::new();
    let steps = 200;

    for i in 0..steps {
        let angle = 2.0 * PI * i as f32 / steps as f32;
        let point = Vector2::new(
            radius * angle.cos(),
            radius * angle.sin(),
        );
        points.push(point);
    }

    points
}

fn create_heart_path(scale: f32) -> Vec<Vector2> {
    let mut points = Vec::new();
    let steps = 300;

    for i in 0..steps {
        let t = 2.0 * PI * i as f32 / steps as f32;
        let x = 16.0 * t.sin().powi(3);
        let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
        let point = Vector2::new(
            x * scale,
            y * scale,
        );
        points.push(point);
    }

    points
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1200, 800)
        .title("Fourier Series Visualization")
        .build();

    let mut time: f32 = 0.0;
    let dt: f32 = 0.0005;
    let num_components = 100;
    let show_circles = true;

    let square_path = create_square_path(150.0);
    let circle_path = create_circle_path(120.0);
    let heart_path = create_heart_path(6.0);

    let square_path2 = create_square_path(130.0);
    let circle_path2 = create_circle_path(110.0);
    let heart_path2 = create_heart_path(5.5);

    let mut paths = vec![
        PathFourier::new(&square_path, Vector2::new(300.0, 200.0), Color::GREEN, num_components),
        PathFourier::new(&circle_path, Vector2::new(600.0, 200.0), Color::BLUE, num_components),
        PathFourier::new(&heart_path, Vector2::new(900.0, 200.0), Color::RED, num_components),
        PathFourier::new(&square_path2, Vector2::new(300.0, 500.0), Color::YELLOW, num_components),
        PathFourier::new(&circle_path2, Vector2::new(600.0, 500.0), Color::MAGENTA, num_components),
        PathFourier::new(&heart_path2, Vector2::new(900.0, 500.0), Color::ORANGE, num_components),
    ];

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        for path in &mut paths {
            if show_circles {
                path.draw_vectors(&mut d, time, true);
            }

            let point = path.evaluate(time);
            path.update_path(point, 2000);
            path.draw_path(&mut d);
        }

        time += dt;
        if time >= 1.0 {
            time = 0.0;
        }
    }
}
