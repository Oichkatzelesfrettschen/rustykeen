#![doc = "Event playback animation system for step-by-step solver visualization"]

use kenken_solver::SolverEvent;
use std::time::{Duration, Instant};

/// Animation state machine for replaying solver events.
///
/// Controls playback of a pre-recorded sequence of solver events with support for:
/// - Play/pause/stop control
/// - Speed adjustment (10ms to 2 seconds per event)
/// - Manual event stepping
/// - Timeline scrubbing to arbitrary positions
/// - Looping and single-play modes
#[derive(Debug)]
pub struct Animator {
    events: Vec<SolverEvent>,
    current_index: usize,
    state: PlaybackState,
    current_time: Duration,
    speed_millis: u64, // milliseconds between events (10 to 2000)
    last_advance: Option<Instant>,
    loop_enabled: bool,
}

/// Current playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Animation is playing (advancing through events).
    Playing,
    /// Animation is paused (no advancement).
    Paused,
    /// Animation has ended (all events consumed).
    Finished,
    /// Animation is stopped and reset to beginning.
    Stopped,
}

impl Animator {
    /// Create a new animator from a sequence of events.
    ///
    /// Starts in `Stopped` state with default speed of 500ms per event.
    pub fn new(events: Vec<SolverEvent>) -> Self {
        Animator {
            events,
            current_index: 0,
            state: PlaybackState::Stopped,
            current_time: Duration::ZERO,
            speed_millis: 500,
            last_advance: None,
            loop_enabled: false,
        }
    }

    /// Start playback from current position.
    pub fn play(&mut self) {
        if self.state != PlaybackState::Finished {
            self.state = PlaybackState::Playing;
            self.last_advance = Some(Instant::now());
        }
    }

    /// Pause playback without losing position.
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
        self.last_advance = None;
    }

    /// Stop and reset to the beginning.
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_index = 0;
        self.current_time = Duration::ZERO;
        self.last_advance = None;
    }

    /// Advance to the next event manually (single step).
    pub fn step(&mut self) -> Option<&SolverEvent> {
        if self.current_index < self.events.len() {
            let event = &self.events[self.current_index];
            self.current_index += 1;
            self.current_time += Duration::from_millis(self.speed_millis);

            if self.current_index >= self.events.len() {
                if self.loop_enabled {
                    self.current_index = 0;
                } else {
                    self.state = PlaybackState::Finished;
                }
            }

            Some(event)
        } else {
            None
        }
    }

    /// Set playback speed in milliseconds per event.
    ///
    /// Clamped to range [10, 2000] (10ms to 2 seconds).
    pub fn set_speed_millis(&mut self, millis: u64) {
        self.speed_millis = millis.clamp(10, 2000);
    }

    /// Get current speed in milliseconds per event.
    pub fn speed_millis(&self) -> u64 {
        self.speed_millis
    }

    /// Jump to a specific event index.
    ///
    /// Returns the event at the new position if valid, None if out of bounds.
    pub fn jump_to(&mut self, index: usize) -> Option<&SolverEvent> {
        if index < self.events.len() {
            self.current_index = index;
            self.current_time = Duration::from_millis((index as u64) * self.speed_millis);
            Some(&self.events[index])
        } else {
            None
        }
    }

    /// Get the current event without advancing.
    pub fn current_event(&self) -> Option<&SolverEvent> {
        if self.current_index < self.events.len() {
            Some(&self.events[self.current_index])
        } else {
            None
        }
    }

    /// Get current playback state.
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get total number of events.
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Get current event index (0-based).
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Get progress as fraction [0.0, 1.0].
    pub fn progress(&self) -> f64 {
        if self.events.is_empty() {
            0.0
        } else {
            (self.current_index as f64) / (self.events.len() as f64)
        }
    }

    /// Set looping mode.
    pub fn set_looping(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
    }

    /// Is looping enabled?
    pub fn is_looping(&self) -> bool {
        self.loop_enabled
    }

    /// Advance animation if playing.
    ///
    /// Returns the next event if one is available and should be displayed,
    /// or None if animation is not playing or events are exhausted.
    pub fn advance(&mut self) -> Option<&SolverEvent> {
        if self.state != PlaybackState::Playing {
            return None;
        }

        let now = Instant::now();
        if let Some(last) = self.last_advance {
            let elapsed = now.duration_since(last);

            if elapsed >= Duration::from_millis(self.speed_millis) {
                self.last_advance = Some(now);

                if self.current_index < self.events.len() {
                    let event = &self.events[self.current_index];
                    self.current_index += 1;

                    if self.current_index >= self.events.len() {
                        if self.loop_enabled {
                            self.current_index = 0;
                        } else {
                            self.state = PlaybackState::Finished;
                        }
                    }

                    return Some(event);
                }
            }
        }

        None
    }

    /// Get all events (read-only).
    pub fn events(&self) -> &[SolverEvent] {
        &self.events
    }

    /// Get remaining events from current position.
    pub fn remaining_events(&self) -> &[SolverEvent] {
        if self.current_index < self.events.len() {
            &self.events[self.current_index..]
        } else {
            &[]
        }
    }

    /// Get elapsed time in animation (scaled by speed).
    pub fn elapsed_time(&self) -> Duration {
        self.current_time
    }

    /// Get estimated total time for full playback.
    pub fn estimated_total_time(&self) -> Duration {
        Duration::from_millis(self.events.len() as u64 * self.speed_millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::CellId;

    fn create_test_events() -> Vec<SolverEvent> {
        vec![
            SolverEvent::Assignment {
                cell: CellId(0),
                value: 1,
                depth: 0,
                deduced: true,
            },
            SolverEvent::Assignment {
                cell: CellId(1),
                value: 2,
                depth: 0,
                deduced: true,
            },
            SolverEvent::SolutionFound {
                total_assignments: 2,
                max_depth: 0,
                nodes_visited: 2,
            },
        ]
    }

    #[test]
    fn animator_creation_and_initial_state() {
        let events = create_test_events();
        let animator = Animator::new(events.clone());

        assert_eq!(animator.state(), PlaybackState::Stopped);
        assert_eq!(animator.current_index(), 0);
        assert_eq!(animator.total_events(), 3);
        assert_eq!(animator.progress(), 0.0);
    }

    #[test]
    fn animator_play_pause_stop() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        animator.play();
        assert_eq!(animator.state(), PlaybackState::Playing);

        animator.pause();
        assert_eq!(animator.state(), PlaybackState::Paused);

        animator.play();
        assert_eq!(animator.state(), PlaybackState::Playing);

        animator.stop();
        assert_eq!(animator.state(), PlaybackState::Stopped);
        assert_eq!(animator.current_index(), 0);
    }

    #[test]
    fn animator_step_advances_index() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        let event1 = animator.step();
        assert!(event1.is_some());
        assert_eq!(animator.current_index(), 1);

        let event2 = animator.step();
        assert!(event2.is_some());
        assert_eq!(animator.current_index(), 2);

        let event3 = animator.step();
        assert!(event3.is_some());
        assert_eq!(animator.current_index(), 3);

        let event4 = animator.step();
        assert!(event4.is_none());
        assert_eq!(animator.state(), PlaybackState::Finished);
    }

    #[test]
    fn animator_speed_clamping() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        animator.set_speed_millis(5); // Too low
        assert_eq!(animator.speed_millis(), 10); // Clamped to minimum

        animator.set_speed_millis(3000); // Too high
        assert_eq!(animator.speed_millis(), 2000); // Clamped to maximum

        animator.set_speed_millis(500);
        assert_eq!(animator.speed_millis(), 500);
    }

    #[test]
    fn animator_jump_to() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        let event = animator.jump_to(1);
        assert!(event.is_some());
        assert_eq!(animator.current_index(), 1);
        assert_eq!(animator.progress(), 1.0 / 3.0);

        let event = animator.jump_to(10); // Out of bounds
        assert!(event.is_none());
        assert_eq!(animator.current_index(), 1); // Index unchanged
    }

    #[test]
    fn animator_progress() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        assert_eq!(animator.progress(), 0.0);

        animator.step();
        assert_eq!(animator.progress(), 1.0 / 3.0);

        animator.step();
        assert_eq!(animator.progress(), 2.0 / 3.0);

        animator.step();
        assert_eq!(animator.progress(), 1.0);
    }

    #[test]
    fn animator_looping() {
        let events = create_test_events();
        let mut animator = Animator::new(events);
        let total_events = animator.total_events();

        animator.set_looping(true);
        assert!(animator.is_looping());

        // Step through one full cycle and verify it wraps.
        for _ in 0..total_events {
            assert!(animator.step().is_some());
        }

        // Should loop back to beginning
        assert_eq!(animator.current_index(), 0);

        animator.set_looping(false);
        for _ in 0..total_events {
            assert!(animator.step().is_some());
        }
        assert_eq!(animator.state(), PlaybackState::Finished);
        assert!(animator.step().is_none());
    }

    #[test]
    fn animator_timing() {
        let events = create_test_events();
        let animator = Animator::new(events);

        assert_eq!(animator.speed_millis(), 500);
        assert_eq!(animator.estimated_total_time(), Duration::from_millis(1500));
    }

    #[test]
    fn animator_current_event() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        let event = animator.current_event();
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type(), "Assignment");

        animator.step();
        let event = animator.current_event();
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type(), "Assignment");
    }

    #[test]
    fn animator_remaining_events() {
        let events = create_test_events();
        let mut animator = Animator::new(events);

        assert_eq!(animator.remaining_events().len(), 3);

        animator.step();
        assert_eq!(animator.remaining_events().len(), 2);

        animator.step();
        assert_eq!(animator.remaining_events().len(), 1);

        animator.step();
        assert_eq!(animator.remaining_events().len(), 0);
    }
}
