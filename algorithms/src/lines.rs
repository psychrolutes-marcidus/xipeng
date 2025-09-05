pub struct SeperateConf {
    pub distance: f32, // meter
    pub time: u32,     // seconds
}

pub struct SeperateConfBuilder {
    distance: Option<f32>,
    time: Option<u32>,
}

impl SeperateConfBuilder {
    pub fn new() -> SeperateConfBuilder {
        SeperateConfBuilder {
            distance: None,
            time: None,
        }
    }

    pub fn distance(&mut self, distance: f32) {
        self.distance = Some(distance)
    }

    pub fn time(&mut self, time: u32) {
        self.time = Some(time)
    }

    pub fn build(&self) -> SeperateConf {
        SeperateConf {
            distance: self.distance.unwrap_or(1000.0),
            time: self.time.unwrap_or(60), // Should be set to whatever we find to be the best value.
        }
    }
}

