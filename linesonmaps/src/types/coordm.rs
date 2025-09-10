use geo_traits::CoordTrait;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoordM {
    pub x: f64,
    pub y: f64,
    pub m: f64,
}

impl CoordM {}

impl From<(f64,f64,f64)> for CoordM{
    fn from((first,second,third): (f64,f64,f64)) -> Self {
        CoordM { x: first, y: second, m: third }
    }
}

impl CoordTrait for CoordM {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xym
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x,
            1 => self.y,
            2 => (self.m),
            e => panic!("tried to access dimension {e} in 3-dimensional coordinate"),
        }
    }
}