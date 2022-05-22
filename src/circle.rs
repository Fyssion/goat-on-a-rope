use fraction::Fraction;

#[derive(Debug)]
pub struct Circle {
    radius: Fraction,
    angle: Fraction,
}

impl Circle {
    pub fn full(radius: Fraction) -> Circle {
        Circle {
            radius,
            angle: Fraction::from(360),
        }
    }

    pub fn partial(radius: Fraction, angle: Fraction) -> Circle {
        Circle { radius, angle }
    }

    /// Returns the area in terms of Pi
    pub fn area(&self) -> Fraction {
        self.partial_area(self.angle)
    }

    pub fn partial_area(&self, angle: Fraction) -> Fraction {
        (self.radius * self.radius) * (angle / Fraction::from(360))
    }
}
