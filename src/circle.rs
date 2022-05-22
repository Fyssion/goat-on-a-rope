#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug)]
pub struct Circle {
    radius: f64,
    angle: f64,
}

impl Circle {
    pub fn full(radius: f64) -> Circle {
        Circle {
            radius,
            angle: 360.0,
        }
    }

    pub fn partial(radius: f64, angle: f64) -> Circle {
        Circle { radius, angle }
    }

    /// Returns the area in terms of Pi
    pub fn area(&self) -> f64 {
        self.partial_area(self.angle)
    }

    pub fn partial_area(&self, angle: f64) -> f64 {
        self.radius.powf(2.0) * (angle / 360.0)
    }
}
