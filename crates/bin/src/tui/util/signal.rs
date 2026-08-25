use std::time::{Duration, Instant};

pub struct DebouncedSignal<T: Clone> {
    value: T,
    pending: Option<(T, Instant, Duration)>,
}

impl<T: Clone> DebouncedSignal<T> {
    pub fn new(value: T, ms: u64) -> Self {
        Self {
            value,
            pending: None,
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T, ms: u64) {
        self.pending = Some((value, Instant::now(), Duration::from_millis(ms)));
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        if let Some((value, start, duration)) = self.pending.take() {
            if now.duration_since(start) >= duration {
                self.value = value;
                return true;
            }
            self.pending = Some((value, start, duration));
        }
        false
    }
}

pub fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

pub struct FadeIn {
    alpha: f64,
    revealed: bool,
    start: Option<Instant>,
}

impl FadeIn {
    pub fn new(show: bool) -> Self {
        Self {
            alpha: if show { 1.0 } else { 0.0 },
            revealed: show,
            start: None,
        }
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn update(&mut self, show: bool, enabled: bool, now: Instant) {
        if !show {
            self.alpha = 0.0;
            self.start = None;
            return;
        }
        if !enabled || self.revealed {
            self.revealed = true;
            self.alpha = 1.0;
            return;
        }
        if self.start.is_none() {
            self.start = Some(now);
            self.alpha = 0.0;
            self.revealed = true;
        }
        if let Some(start) = self.start {
            let elapsed = now.duration_since(start).as_millis() as f64;
            let progress = (elapsed / 160.0).min(1.0);
            self.alpha = smoothstep(progress);
            if progress >= 1.0 {
                self.start = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounced_signal() {
        let mut signal = DebouncedSignal::new(0, 100);
        signal.set(42, 100);
        assert_eq!(*signal.get(), 0);
        let now = Instant::now();
        assert!(!signal.tick(now));
        assert!(signal.tick(now + Duration::from_millis(101)));
        assert_eq!(*signal.get(), 42);
    }

    #[test]
    fn test_fade_in_visible() {
        let mut fade = FadeIn::new(true);
        assert_eq!(fade.alpha(), 1.0);
    }

    #[test]
    fn test_fade_in_hidden() {
        let mut fade = FadeIn::new(false);
        assert_eq!(fade.alpha(), 0.0);
        let now = Instant::now();
        fade.update(true, false, now);
        assert_eq!(fade.alpha(), 1.0);
    }

    #[test]
    fn test_fade_in_animate() {
        let mut fade = FadeIn::new(false);
        let now = Instant::now();
        fade.update(true, true, now);
        assert_eq!(fade.alpha(), 0.0);
        fade.update(true, true, now + Duration::from_millis(80));
        assert!(fade.alpha() > 0.0 && fade.alpha() < 1.0);
        fade.update(true, true, now + Duration::from_millis(160));
        assert_eq!(fade.alpha(), 1.0);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert!(smoothstep(0.5) > 0.0 && smoothstep(0.5) < 1.0);
    }
}
