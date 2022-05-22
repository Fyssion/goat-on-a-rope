use crate::circle::Circle;
use eframe::{egui, epi};
use egui::epaint::PathShape;
use fraction::{Fraction, GenericDecimal};
use std::f32::consts::PI;

type Decimal = GenericDecimal<u64, u8>;

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
    sides: u32,
    position: f32,
    // // this how you opt-out of serialization of a member
    // #[cfg_attr(feature = "persistence", serde(skip))]
    // value: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            sides: 3,
            position: 0.0,
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
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 frame.quit();
        //             }
        //         });
        //     });
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Goat on a Rope");
            // ui.hyperlink("https://github.com/emilk/eframe_template");
            // ui.add(egui::github_link_file!(
            //     "https://github.com/emilk/eframe_template/blob/master/",
            //     "Source code."
            // ));

            ui.add(egui::Slider::new(&mut self.sides, 3..=20).text("sides"));
            ui.add(egui::Slider::new(&mut self.position, 0.0..=1.0).text("position"));

            let outside_angle = Fraction::from(360) / Fraction::from(self.sides);
            let inside_angle = Fraction::from(180) - outside_angle;

            let on_vertex = self.position == 0.0 || self.position == 1.0;
            let initial_angle = if on_vertex {
                Fraction::from(360) - inside_angle
            } else {
                Fraction::from(180)
            };

            println!("Calculating");

            println!("Pushing circle {:?}", Circle::partial(Fraction::from(1), initial_angle));
            let mut circles = vec![Circle::partial(Fraction::from(1), initial_angle)];

            if on_vertex {
                println!("Calculating left side");
                self.calculate_circles(&mut circles, Fraction::from(1));
                println!("Calculating right side");
                self.calculate_circles(&mut circles, Fraction::from(1));
            } else {
                self.calculate_circles(&mut circles, Fraction::from(self.position));
                self.calculate_circles(&mut circles, Fraction::from(1) - Fraction::from(self.position));
            }
            let mut area = Fraction::from(0);
            let circle_count = circles.len();

            for circle in circles {
                area += circle.area();
            }

            ui.label(format!("Circles: {}", circle_count));
            ui.label(format!("Area: {}", area));
            ui.label(format!("Decimal: {:.5}", Decimal::from_fraction(area)));

            ui.separator();

            // egui::Frame::canvas(ui.style()).show(ui, |ui| {
            //     self.draw_graphic(ui);
            // });
            self.draw_graphic(ui);

            ui.separator();
            egui::warn_if_debug_build(ui);
        });
    }
}

impl App {
    pub fn calculate_circles(&self, circles: &mut Vec<Circle>, position: Fraction) {
        let outside_angle = Fraction::from(360) / Fraction::from(self.sides);
        let side_length = Fraction::from(2) / Fraction::from(self.sides);

        let mut radius = Fraction::from(1) - (position * side_length);
        let circle = Circle::partial(radius, outside_angle);
        println!("Pushing circle {:?}", circle);
        circles.push(circle);
        radius -= side_length;

        println!("Side length: {} | Radius: {}", side_length, radius);

        while radius > Fraction::from(0) {
            let circle = Circle::partial(radius, outside_angle);
            println!("Pushing circle {:?}", circle);
            circles.push(circle);
            radius -= side_length;
        }
    }

    pub fn draw_graphic(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(egui::Vec2::new(ui.available_width(), 300.0), egui::Sense::hover());

        self.paint_polygon(&painter, self.sides);

        response
    }

    pub fn paint_polygon(&self, painter: &egui::Painter, sides: u32) {
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 200, 100));
        let mut points: Vec<egui::Pos2> = Vec::new();

        let outside_angle = 360.0 / self.sides as f32;

        let center = egui::pos2(150.0, 300.0);
        let mut angle = 90.0;

        for _ in 0..=sides {
            let point = self.calculate_other_point(center, angle, 75.0);
            points.push(point);
            angle += outside_angle;
        }

        painter.add(PathShape::line(points, stroke));
    }

    fn calculate_other_point(&self, original: egui::Pos2, angle: f32, distance: f32) -> egui::Pos2 {
        let angle_rad = App::to_rad(angle);
        let diff = egui::vec2(distance * angle_rad.cos(), distance * angle_rad.sin());
        original + diff
    }

    fn to_rad(angle: f32) -> f32 {
        angle * (PI / 180.0)
    }
}
