#![deny(unsafe_code)]

use super::debouncer::{Debouncer, Edge};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PinState {
    Low,
    High,
}

#[derive(Debug)]
pub struct SmallPinDebouncer {
    inner: Debouncer<PinState, u8>,
}

impl SmallPinDebouncer {
    pub fn new(threshold: u8, inital_state: PinState) -> Self {
        SmallPinDebouncer {
            inner: Debouncer::new(threshold, inital_state),
        }
    }

    pub fn update(&mut self, state: PinState) -> Option<Edge<PinState>> {
        self.inner.update(state)
    }

    pub fn is_high(&self) -> bool {
        self.inner.is_state(PinState::High)
    }

    pub fn is_low(&self) -> bool {
        self.inner.is_state(PinState::Low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rising_edge() {
        // Initially low state
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(3, PinState::Low);
        assert!(debouncer.is_low());

        // Three high updates required
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );

        // Further highs do not indicate a rising edge anymore
        assert_eq!(debouncer.update(PinState::High), None);

        // A low state does not reset counting
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_ne!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
    }

    #[test]
    fn test_falling_edge() {
        // Initially high state
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(3, PinState::High);
        assert!(debouncer.is_high());

        // Three low updates required
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );

        // Further lowss do not indicate a rising edge anymore
        assert_eq!(debouncer.update(PinState::Low), None);

        // A high state does not reset counting
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_ne!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
    }

    #[test]
    fn test_debounce_16() {
        // Sixteen pressed updates required
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(16, PinState::Low);
        assert!(debouncer.is_low());
        for _ in 0..15 {
            assert_eq!(debouncer.update(PinState::High), None);
            assert!(!debouncer.is_high());
        }
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert!(debouncer.is_high());
        assert_eq!(debouncer.update(PinState::High), None);
        assert!(debouncer.is_high());
    }

    #[test]
    fn test_is_low_high() {
        // Initially low
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(8, PinState::Low);
        assert!(debouncer.is_low());
        assert!(!debouncer.is_high());

        // Depressed updates don't change the situation
        debouncer.update(PinState::Low);
        assert!(debouncer.is_low());
        assert!(!debouncer.is_high());

        // A pressed update causes neither low nor high state
        for _ in 0..7 {
            assert!(debouncer.update(PinState::High).is_none());
            assert!(!debouncer.is_low());
            assert!(!debouncer.is_high());
        }

        // Once complete, the state is high
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert!(!debouncer.is_low());
        assert!(debouncer.is_high());

        // Consecutive pressed updates don't trigger an edge but are still high
        assert!(debouncer.update(PinState::High).is_none());
        assert!(!debouncer.is_low());
        assert!(debouncer.is_high());
    }

    /// Ensure the promised low RAM consumption.
    #[test]
    fn test_ram_consumption() {
        // Regular debouncers
        assert_eq!(
            std::mem::size_of_val(&SmallPinDebouncer::new(2, PinState::Low)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&SmallPinDebouncer::new(8, PinState::Low)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&SmallPinDebouncer::new(9, PinState::Low)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&SmallPinDebouncer::new(16, PinState::Low)),
            4
        );
    }

    /// Ensure that the initial state can be specified.
    #[test]
    fn test_initial_state() {
        let mut debouncer_01 = SmallPinDebouncer::new(2, PinState::Low);
        assert_eq!(debouncer_01.update(PinState::Low), None);
        assert_eq!(debouncer_01.update(PinState::Low), None);
        assert_eq!(debouncer_01.update(PinState::High), None);
        assert_eq!(
            debouncer_01.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );

        let mut debouncer_02 = SmallPinDebouncer::new(2, PinState::Low);
        assert_eq!(debouncer_02.update(PinState::High), None);
        assert_eq!(
            debouncer_02.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer_02.update(PinState::Low), None);
        assert_eq!(
            debouncer_02.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );

        let mut debouncer_03 = SmallPinDebouncer::new(2, PinState::High);
        assert_eq!(debouncer_03.update(PinState::Low), None);
        assert_eq!(
            debouncer_03.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer_03.update(PinState::High), None);
        assert_eq!(
            debouncer_03.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );

        let mut debouncer_04 = SmallPinDebouncer::new(2, PinState::High);
        assert_eq!(debouncer_04.update(PinState::High), None);
        assert_eq!(debouncer_04.update(PinState::High), None);
        assert_eq!(debouncer_04.update(PinState::Low), None);
        assert_eq!(
            debouncer_04.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
    }

    #[test]
    fn test_long_running_02() {
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(2, PinState::Low);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
    }

    #[test]
    fn test_long_running_04() {
        let mut debouncer: SmallPinDebouncer = SmallPinDebouncer::new(4, PinState::Low);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(
            debouncer.update(PinState::High),
            Some(Edge::new(PinState::Low, PinState::High))
        );
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(
            debouncer.update(PinState::Low),
            Some(Edge::new(PinState::High, PinState::Low))
        );
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
        assert_eq!(debouncer.update(PinState::Low), None);
        assert_eq!(debouncer.update(PinState::High), None);
    }
}
