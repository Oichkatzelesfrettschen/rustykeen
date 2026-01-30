// Propagation ripple animation effects for visualizing constraint propagation

use std::time::{Duration, Instant};
use kenken_core::CellId;

/// Single ripple emanating from a cell.
#[derive(Debug, Clone)]
pub struct Ripple {
    /// Origin cell of the ripple
    origin: CellId,
    /// When ripple was created
    start_time: Instant,
    /// Duration for full ripple expansion
    duration: Duration,
    /// Maximum radius in pixels
    max_radius: f64,
    /// Intensity (0.0 to 1.0)
    intensity: f64,
}

impl Ripple {
    /// Create new ripple at cell.
    pub fn new(origin: CellId, duration: Duration, max_radius: f64) -> Self {
        Ripple {
            origin,
            start_time: Instant::now(),
            duration,
            max_radius,
            intensity: 1.0,
        }
    }

    /// Get origin cell.
    pub fn origin(&self) -> CellId {
        self.origin
    }

    /// Get current progress (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            1.0
        } else {
            elapsed.as_secs_f64() / self.duration.as_secs_f64()
        }
    }

    /// Check if ripple has completed.
    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    /// Get current radius.
    pub fn radius(&self) -> f64 {
        self.max_radius * self.progress()
    }

    /// Get current visual intensity (decreases with progress).
    pub fn visual_intensity(&self) -> f64 {
        let progress = self.progress();
        self.intensity * (1.0 - progress)
    }

    /// Set intensity multiplier.
    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.max(0.0).min(1.0);
    }
}

/// Manager for multiple concurrent ripples.
#[derive(Debug, Default)]
pub struct RippleAnimator {
    ripples: Vec<Ripple>,
}

impl RippleAnimator {
    /// Create empty ripple animator.
    pub fn new() -> Self {
        RippleAnimator {
            ripples: Vec::new(),
        }
    }

    /// Add new ripple at cell.
    pub fn add_ripple(&mut self, cell: CellId) {
        let ripple = Ripple::new(cell, Duration::from_millis(500), 80.0);
        self.ripples.push(ripple);
    }

    /// Add ripple with custom parameters.
    pub fn add_ripple_custom(&mut self, cell: CellId, duration: Duration, max_radius: f64, intensity: f64) {
        let mut ripple = Ripple::new(cell, duration, max_radius);
        ripple.set_intensity(intensity);
        self.ripples.push(ripple);
    }

    /// Update ripples and remove finished ones.
    pub fn update(&mut self) {
        self.ripples.retain(|r| !r.is_finished());
    }

    /// Get active ripples.
    pub fn ripples(&self) -> &[Ripple] {
        &self.ripples
    }

    /// Get mutable ripples.
    pub fn ripples_mut(&mut self) -> &mut [Ripple] {
        &mut self.ripples
    }

    /// Clear all ripples.
    pub fn clear(&mut self) {
        self.ripples.clear();
    }

    /// Get number of active ripples.
    pub fn count(&self) -> usize {
        self.ripples.len()
    }

    /// Check if any ripples are active.
    pub fn is_animating(&self) -> bool {
        !self.ripples.is_empty()
    }
}

/// Ripple burst with multiple concentric rings.
#[derive(Debug)]
pub struct RippleBurst {
    /// Center cell
    center: CellId,
    /// Ripples in this burst (staggered timing)
    ripples: Vec<Ripple>,
}

impl RippleBurst {
    /// Create burst with N concentric ripples.
    pub fn new(center: CellId, ring_count: usize, base_duration: Duration, base_radius: f64) -> Self {
        let mut ripples = Vec::new();

        for i in 0..ring_count {
            let stagger = Duration::from_millis((i as u64) * 100);
            let mut ripple = Ripple::new(
                center,
                base_duration + stagger,
                base_radius * ((i + 1) as f64),
            );

            // Outer rings are dimmer
            let intensity = 1.0 - ((i as f64) / (ring_count as f64)) * 0.5;
            ripple.set_intensity(intensity);

            ripples.push(ripple);
        }

        RippleBurst { center, ripples }
    }

    /// Get center cell.
    pub fn center(&self) -> CellId {
        self.center
    }

    /// Update burst ripples.
    pub fn update(&mut self) {
        self.ripples.retain(|r| !r.is_finished());
    }

    /// Check if burst is complete.
    pub fn is_finished(&self) -> bool {
        self.ripples.is_empty()
    }

    /// Get ripples in burst.
    pub fn ripples(&self) -> &[Ripple] {
        &self.ripples
    }

    /// Count remaining ripples.
    pub fn ripple_count(&self) -> usize {
        self.ripples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn ripple_creation() {
        let ripple = Ripple::new(CellId(0), Duration::from_millis(500), 80.0);
        assert_eq!(ripple.origin(), CellId(0));
        assert!(!ripple.is_finished());
        assert!(ripple.progress() < 0.01); // Very small, due to test execution time
    }

    #[test]
    fn ripple_progress() {
        let ripple = Ripple::new(CellId(0), Duration::from_millis(100), 80.0);
        let progress = ripple.progress();
        assert!(progress >= 0.0 && progress <= 1.0);
    }

    #[test]
    fn ripple_radius() {
        let ripple = Ripple::new(CellId(0), Duration::from_millis(500), 100.0);
        let radius = ripple.radius();
        assert!(radius > 0.0 && radius <= 100.0);
    }

    #[test]
    fn ripple_intensity_decreases() {
        let ripple = Ripple::new(CellId(0), Duration::from_millis(500), 80.0);
        let initial = ripple.visual_intensity();

        thread::sleep(Duration::from_millis(100));

        let ripple2 = Ripple::new(CellId(0), Duration::from_millis(500), 80.0);
        let later = ripple2.visual_intensity();

        // Initial ripple should have higher intensity
        assert!(initial > later);
    }

    #[test]
    fn ripple_animator_add_ripple() {
        let mut animator = RippleAnimator::new();
        assert_eq!(animator.count(), 0);

        animator.add_ripple(CellId(0));
        assert_eq!(animator.count(), 1);

        animator.add_ripple(CellId(1));
        assert_eq!(animator.count(), 2);
    }

    #[test]
    fn ripple_animator_update() {
        let mut animator = RippleAnimator::new();
        animator.add_ripple(CellId(0));
        assert!(animator.is_animating());

        // Manually set ripple as finished for testing
        animator.ripples_mut()[0].start_time = Instant::now() - Duration::from_secs(10);

        animator.update();
        assert!(!animator.is_animating());
    }

    #[test]
    fn ripple_animator_clear() {
        let mut animator = RippleAnimator::new();
        animator.add_ripple(CellId(0));
        animator.add_ripple(CellId(1));
        assert_eq!(animator.count(), 2);

        animator.clear();
        assert_eq!(animator.count(), 0);
    }

    #[test]
    fn ripple_burst_creation() {
        let burst = RippleBurst::new(CellId(5), 3, Duration::from_millis(500), 50.0);
        assert_eq!(burst.center(), CellId(5));
        assert_eq!(burst.ripple_count(), 3);
    }

    #[test]
    fn ripple_burst_concentric() {
        let burst = RippleBurst::new(CellId(0), 3, Duration::from_millis(500), 50.0);
        let ripples = burst.ripples();

        // Check that radii increase for each ring
        assert!(ripples[0].max_radius < ripples[1].max_radius);
        assert!(ripples[1].max_radius < ripples[2].max_radius);
    }

    #[test]
    fn ripple_intensity_control() {
        let mut ripple = Ripple::new(CellId(0), Duration::from_millis(500), 80.0);
        ripple.set_intensity(0.5);
        let intensity = ripple.visual_intensity();
        assert!((intensity - 0.5).abs() < 0.001); // Allow for float rounding

        ripple.set_intensity(1.5); // Clamps to 1.0
        let intensity = ripple.visual_intensity();
        assert!(intensity <= 1.0);

        ripple.set_intensity(-0.5); // Clamps to 0.0
        assert_eq!(ripple.visual_intensity(), 0.0);
    }
}
