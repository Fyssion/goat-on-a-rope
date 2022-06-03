use crate::circle::Circle;
use eframe::{egui, epi};
use egui::epaint::PathShape;
use fraction::{Fraction, GenericDecimal};
use std::f32::consts::PI;
use std::f64::consts::TAU;

type Decimal = GenericDecimal<u64, u8>;

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    sides: u32,
    position: f32,

    trim_circles: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            sides: 3,
            position: 0.0,
            trim_circles: true,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Goat on a Rope"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `Sf32::consts::PIu.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Goat on a Rope");
            // ui.hyperlink("https://github.com/emilk/eframe_template");
            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/master/",
            //     "Source code."
            // ));

            ui.add(egui::Slider::new(&mut self.sides, 3..=20).text("sides"));
            ui.add(egui::Slider::new(&mut self.position, 0.0..=0.99).text("position"));

            let outside_angle = Fraction::from(360) / Fraction::from(self.sides);
            let inside_angle = Fraction::from(180) - outside_angle;

            let on_vertex = self.position == 0.0 || self.position == 1.0;
            let initial_angle = if on_vertex {
                Fraction::from(360) - inside_angle
            } else {
                Fraction::from(180)
            };

            let mut calculations = vec![];

            let circle = Circle::partial(Fraction::from(1), initial_angle);
            calculations.push(circle.area_formula());
            let mut circles = vec![Circle::partial(Fraction::from(1), initial_angle)];
            let left_count;

            if on_vertex {
                left_count =
                    self.calculate_circles(&mut circles, Fraction::from(1), &mut calculations);
                self.calculate_circles(&mut circles, Fraction::from(1), &mut calculations);
            } else {
                left_count = self.calculate_circles(
                    &mut circles,
                    Fraction::from(self.position),
                    &mut calculations,
                );
                self.calculate_circles(
                    &mut circles,
                    Fraction::from(1) - Fraction::from(self.position),
                    &mut calculations,
                );
            }
            let mut area = Fraction::from(0);
            let circle_count = circles.len();

            for circle in circles.iter() {
                area += circle.area();
            }

            ui.label(format!("Circles: {}", circle_count));
            ui.label(calculations.join(" + "));
            ui.label(format!("Area: {}", area));
            ui.label(format!("Decimal: {:.5}", Decimal::from_fraction(area)));

            ui.checkbox(&mut self.trim_circles, "Trim Circles");

            ui.separator();

            self.draw_graphic(ui, circles, left_count);

            ui.separator();
            egui::warn_if_debug_build(ui);
        });
    }
}

impl App {
    pub fn calculate_circles(
        &self,
        circles: &mut Vec<Circle>,
        position: Fraction,
        calculations: &mut Vec<String>,
    ) -> u32 {
        let outside_angle = Fraction::from(360) / Fraction::from(self.sides);
        let side_length = Fraction::from(2) / Fraction::from(self.sides);

        let mut radius = Fraction::from(1) - (position * side_length);
        let circle = Circle::partial(radius, outside_angle);
        calculations.push(circle.area_formula());
        circles.push(circle);
        radius -= side_length;

        let mut circle_count = 1;

        while radius > Fraction::from(0) {
            let circle = Circle::partial(radius, outside_angle);
            calculations.push(circle.area_formula());
            circles.push(circle);
            circle_count += 1;
            radius -= side_length;
        }

        circle_count
    }

    pub fn draw_graphic(
        &mut self,
        ui: &mut egui::Ui,
        circles: Vec<Circle>,
        left_count: u32,
    ) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), 400.0),
            egui::Sense::hover(),
        );

        let vertices = self.paint_polygon(&painter, self.sides);

        let outside_angle = 360.0 / self.sides as f32;
        let inside_angle = 180.0 - outside_angle;
        let side_length =
            50.0 * App::to_rad(outside_angle).sin() / App::to_rad(inside_angle / 2.0).sin();
        let on_vertex = self.position == 0.0 || self.position == 1.0;
        let circle_middle_angle = if on_vertex {
            90.0
        } else {
            (90.0 + (90.0 - outside_angle)) / 2.0
        };

        let starting_point = if on_vertex {
            self.calculate_other_point(
                vertices[0],
                self.add_angle(360.0, -outside_angle / 2.0),
                side_length * self.position,
            )
        } else {
            self.calculate_other_point(
                vertices[0],
                self.add_angle(360.0, -outside_angle / 2.0),
                side_length * (1.0 - self.position),
            )
        };
        let radius = side_length * self.sides as f32 / 2.0;
        let angle =
            *circles[0].angle.numer().unwrap() as f32 / *circles[0].angle.denom().unwrap() as f32;

        let mut prev_left_angle = self.add_angle(circle_middle_angle, -angle / 2.0);
        let mut left_angle;
        let mut prev_right_angle = self.add_angle(circle_middle_angle, angle / 2.0);
        let mut right_angle;
        let color = egui::Color32::RED;
        let first_circle = if self.trim_circles {
            self.circle(
                starting_point,
                radius,
                prev_left_angle,
                prev_right_angle,
                color,
            )
        } else {
            self.circle(starting_point, radius, 0.0, 360.0, color)
        };
        painter.add(first_circle);

        let center = egui::pos2(200.0, 300.0);
        let mut angle = 90.0;

        let colors = vec![
            egui::Color32::from_rgb(255, 165, 0),
            egui::Color32::YELLOW,
            egui::Color32::LIGHT_BLUE,
            egui::Color32::BLUE,
            egui::Color32::from_rgb(138, 43, 226),
        ];

        let mut using_colors = colors.clone();

        for circle in (&circles[1..=left_count as usize]).iter() {
            angle -= outside_angle;
            let point = self.calculate_other_point(center, angle, 50.0);
            let new_radius = (*circle.radius.numer().unwrap() as f32
                / *circle.radius.denom().unwrap() as f32)
                * radius;
            left_angle = prev_left_angle;
            prev_left_angle = self.add_angle(prev_left_angle, -outside_angle);
            let shape = if self.trim_circles {
                let pre = self.circle(
                    point,
                    new_radius,
                    prev_left_angle,
                    left_angle,
                    using_colors[0],
                );
                let mut path = pre.points.clone();
                path.push(point);
                PathShape::line(path, pre.stroke)
            } else {
                self.circle(point, new_radius, 0.0, 360.0, using_colors[0])
            };
            painter.add(shape);
            using_colors.remove(0);
            if using_colors.is_empty() {
                using_colors = colors.clone();
            }
        }

        let center = egui::pos2(200.0, 300.0);
        let mut angle = 90.0;
        let on_vertex = self.position == 0.0 || self.position == 1.0;
        using_colors = colors.clone();

        for circle in (&circles[left_count as usize + 1..]).iter() {
            if on_vertex {
                angle += outside_angle;
            }
            let point = self.calculate_other_point(center, angle, 50.0);
            let new_radius = (*circle.radius.numer().unwrap() as f32
                / *circle.radius.denom().unwrap() as f32)
                * radius;
            right_angle = prev_right_angle;
            prev_right_angle = self.add_angle(prev_right_angle, outside_angle);
            let shape = if self.trim_circles {
                let pre = self.circle(
                    point,
                    new_radius,
                    right_angle,
                    prev_right_angle,
                    using_colors[0],
                );
                let mut path = pre.points.clone();
                path.insert(0, point);
                PathShape::line(path, pre.stroke)
            } else {
                self.circle(point, new_radius, 0.0, 360.0, using_colors[0])
            };
            painter.add(shape);
            if !on_vertex {
                angle += outside_angle;
            }

            using_colors.remove(0);
            if using_colors.is_empty() {
                using_colors = colors.clone();
            }
        }

        response
    }

    pub fn paint_polygon(&self, painter: &egui::Painter, sides: u32) -> Vec<egui::Pos2> {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 200, 100));
        let mut points: Vec<egui::Pos2> = Vec::new();

        let outside_angle = 360.0 / self.sides as f32;

        let center = egui::pos2(200.0, 300.0);
        let mut angle = 90.0;

        for _ in 0..=sides {
            let point = self.calculate_other_point(center, angle, 50.0);
            points.push(point);
            angle += outside_angle;
        }

        painter.add(PathShape::line(points.clone(), stroke));

        points
    }

    fn calculate_other_point(&self, original: egui::Pos2, angle: f32, distance: f32) -> egui::Pos2 {
        let angle_rad = App::to_rad(angle);
        let diff = egui::vec2(distance * angle_rad.cos(), distance * angle_rad.sin());
        original + diff
    }

    fn to_rad(angle: f32) -> f32 {
        angle * (PI / 180.0)
    }

    fn circle(
        &self,
        center: egui::Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: egui::Color32,
    ) -> PathShape {
        let stroke = egui::Stroke::new(1.0, color);
        let n = 512;
        let start = ((start_angle / 360.0) * n as f32) as usize;
        let end = ((end_angle / 360.0) * n as f32) as usize;
        let range = if start_angle > end_angle {
            let mut first = (start as usize..=512).collect::<Vec<_>>();
            let second = (0..=end as usize).collect::<Vec<_>>();
            first.extend(second);
            first
        } else {
            ((start as usize)..=(end as usize)).collect::<Vec<_>>()
        };

        let mut circle: Vec<egui::Pos2> = Vec::new();
        for i in range {
            let t = egui::remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
            let r = radius as f64;
            circle.push(egui::pos2(
                (r * t.cos() + center.x as f64) as f32,
                (r * t.sin() + center.y as f64) as f32,
            ));
        }
        PathShape::line(circle, stroke)
    }

    fn add_angle(&self, angle: f32, other: f32) -> f32 {
        if (angle + other) < 0.0 {
            360.0 + other + angle
        } else if (angle + other) > 360.0 {
            0.0 + other - (360.0 - angle)
        } else {
            angle + other
        }
    }
}
