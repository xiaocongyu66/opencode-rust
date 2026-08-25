use std::time::Instant;

pub trait ScrollAcceleration {
    fn tick(&mut self) -> f64;
    fn reset(&mut self);
}

pub struct CustomSpeedScroll {
    speed: f64,
}

impl CustomSpeedScroll {
    pub fn new(speed: f64) -> Self {
        Self { speed }
    }
}

impl ScrollAcceleration for CustomSpeedScroll {
    fn tick(&mut self) -> f64 {
        self.speed
    }

    fn reset(&mut self) {}
}

/// macOS-style scroll acceleration: rapid consecutive wheel events build up
/// velocity (so fast scrolling covers more ground), then the velocity decays
/// exponentially once input stops. This gives the inertial feel of a trackpad
/// rather than the flat fixed-step scroll of a typical terminal.
pub struct MacOSScrollAccel {
    velocity: f64,
    last_ts: Option<Instant>,
}

impl MacOSScrollAccel {
    pub fn new() -> Self {
        Self { velocity: 0.0, last_ts: None }
    }

    /// Feed one wheel "tick" (unit impulse). Each call adds to the accumulated
    /// velocity; calling tick() without feeding new input lets it decay.
    pub fn feed(&mut self, impulse: f64) {
        let now = Instant::now();
        // If the previous event was more than 120ms ago, the user paused —
        // reset velocity so a new flick starts clean instead of inheriting
        // stale momentum.
        if let Some(prev) = self.last_ts {
            if now.duration_since(prev).as_millis() > 120 {
                self.velocity = 0.0;
            }
        }
        self.velocity += impulse.abs();
        self.last_ts = Some(now);
    }
}

impl ScrollAcceleration for MacOSScrollAccel {
    fn tick(&mut self) -> f64 {
        // Decay: each tick multiplies velocity by 0.85. Below the floor we
        // snap to zero so a resting view doesn't jitter by a fraction of a
        // line forever.
        self.velocity *= 0.85;
        if self.velocity < 0.5 {
            self.velocity = 0.0;
        }
        self.velocity
    }

    fn reset(&mut self) {
        self.velocity = 0.0;
        self.last_ts = None;
    }
}

impl Default for MacOSScrollAccel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ScrollConfig {
    pub scroll_acceleration_enabled: Option<bool>,
    pub scroll_speed: Option<f64>,
}

pub fn get_scroll_acceleration(config: Option<&ScrollConfig>) -> Box<dyn ScrollAcceleration> {
    if let Some(c) = config {
        if c.scroll_acceleration_enabled == Some(true) {
            return Box::new(MacOSScrollAccel::new());
        }
        if let Some(speed) = c.scroll_speed {
            return Box::new(CustomSpeedScroll::new(speed));
        }
    }
    Box::new(CustomSpeedScroll::new(3.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_speed() {
        let mut s = CustomSpeedScroll::new(5.0);
        assert_eq!(s.tick(), 5.0);
        s.reset();
        assert_eq!(s.tick(), 5.0);
    }

    #[test]
    fn test_default_config() {
        let mut s = get_scroll_acceleration(None);
        assert_eq!(s.tick(), 3.0);
    }

    #[test]
    fn test_acceleration_enabled() {
        let config = ScrollConfig {
            scroll_acceleration_enabled: Some(true),
            scroll_speed: None,
        };
        // get_scroll_acceleration returns a MacOSScrollAccel when enabled.
        // Verify the acceleration behavior: idle → 0, after feed → decays.
        let mut s = get_scroll_acceleration(Some(&config));
        assert_eq!(s.tick(), 0.0);
        let mut m = MacOSScrollAccel::new();
        m.feed(5.0);
        let v1 = m.tick();
        assert!(v1 > 0.0);
        let v2 = m.tick();
        assert!(v2 < v1);
    }

    #[test]
    fn test_custom_speed_from_config() {
        let config = ScrollConfig {
            scroll_acceleration_enabled: Some(false),
            scroll_speed: Some(10.0),
        };
        let mut s = get_scroll_acceleration(Some(&config));
        assert_eq!(s.tick(), 10.0);
    }
}
