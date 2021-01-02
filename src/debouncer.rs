#![deny(unsafe_code)]

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Edge<T> {
    from: T,
    to: T,
}

impl<T> Edge<T> {
    pub fn new(from: T, to: T) -> Self {
        Edge { from, to }
    }
}

#[derive(Debug)]
pub struct Debouncer<T, S> {
    current_state: T,
    next_state: T,
    repetition_count: S,
    threshold: S,
}

impl<T, S> Debouncer<T, S>
where
    T: PartialEq + Copy,
    S: num::traits::One + core::ops::Add<Output = S> + PartialEq + PartialOrd + Copy,
{
    pub fn new(threshold: S, inital_state: T) -> Self {
        Debouncer {
            current_state: inital_state,
            next_state: inital_state,
            repetition_count: threshold,
            threshold: threshold,
        }
    }

    pub fn update(&mut self, state: T) -> Option<Edge<T>> {
        if self.current_state == state {
            self.current_state = self.current_state;
            self.next_state = state;
            self.repetition_count = self.repetition_count;

            None
        } else if self.current_state != state && self.next_state != state {
            self.current_state = self.current_state;
            self.next_state = state;
            self.repetition_count = S::one();

            None
        } else if self.current_state != state
            && self.next_state == state
            && self.repetition_count + S::one() < self.threshold
        {
            self.current_state = self.current_state;
            self.next_state = state;
            self.repetition_count = self.repetition_count + S::one();

            None
        } else if self.current_state != state
            && self.next_state == state
            && self.repetition_count + S::one() >= self.threshold
        {
            let from_state = self.current_state;
            let to_state = self.next_state;

            self.current_state = state;
            self.next_state = state;
            self.repetition_count = self.threshold;

            Some(Edge::new(from_state, to_state))
        } else {
            // Only so that the compiler does not complain
            None
        }
    }

    pub fn is_state(&self, state: T) -> bool {
        self.current_state == self.next_state && self.current_state == state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, PartialEq, Clone, Copy)]
    enum ABState {
        A,
        B,
    }

    #[derive(Debug)]
    struct ABDebouncer {
        inner: Debouncer<ABState, u8>,
    }
    
    impl ABDebouncer {
        fn new(threshold: u8, inital_state: ABState) -> Self {
            ABDebouncer {
                inner: Debouncer::new(threshold, inital_state),
            }
        }
    
        fn update(&mut self, state: ABState) -> Option<Edge<ABState>> {
            self.inner.update(state)
        }
    
        fn is_a(&self) -> bool {
            self.inner.is_state(ABState::A)
        }
    
        fn is_b(&self) -> bool {
            self.inner.is_state(ABState::B)
        }
    }

    #[test]
    fn test_rising_edge() {
        // Initially low state
        let mut debouncer: ABDebouncer = ABDebouncer::new(3, ABState::A);
        assert!(debouncer.is_a());

        // Three high updates required
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );

        // Further highs do not indicate a rising edge anymore
        assert_eq!(debouncer.update(ABState::B), None);

        // A low state does not reset counting
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_ne!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
    }

    #[test]
    fn test_falling_edge() {
        // Initially high state
        let mut debouncer: ABDebouncer = ABDebouncer::new(3, ABState::B);
        assert!(debouncer.is_b());

        // Three low updates required
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );

        // Further lowss do not indicate a rising edge anymore
        assert_eq!(debouncer.update(ABState::A), None);

        // A high state does not reset counting
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_ne!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
    }

    #[test]
    fn test_debounce_16() {
        // Sixteen pressed updates required
        let mut debouncer: ABDebouncer = ABDebouncer::new(16, ABState::A);
        assert!(debouncer.is_a());
        for _ in 0..15 {
            assert_eq!(debouncer.update(ABState::B), None);
            assert!(!debouncer.is_b());
        }
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert!(debouncer.is_b());
        assert_eq!(debouncer.update(ABState::B), None);
        assert!(debouncer.is_b());
    }

    #[test]
    fn test_is_a_high() {
        // Initially low
        let mut debouncer: ABDebouncer = ABDebouncer::new(8, ABState::A);
        assert!(debouncer.is_a());
        assert!(!debouncer.is_b());

        // Depressed updates don't change the situation
        debouncer.update(ABState::A);
        assert!(debouncer.is_a());
        assert!(!debouncer.is_b());

        // A pressed update causes neither low nor high state
        for _ in 0..7 {
            assert!(debouncer.update(ABState::B).is_none());
            assert!(!debouncer.is_a());
            assert!(!debouncer.is_b());
        }

        // Once complete, the state is high
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert!(!debouncer.is_a());
        assert!(debouncer.is_b());

        // Consecutive pressed updates don't trigger an edge but are still high
        assert!(debouncer.update(ABState::B).is_none());
        assert!(!debouncer.is_a());
        assert!(debouncer.is_b());
    }

    /// Ensure the promised low RAM consumption.
    #[test]
    fn test_ram_consumption() {
        // Regular debouncers
        assert_eq!(
            std::mem::size_of_val(&ABDebouncer::new(2, ABState::A)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&ABDebouncer::new(8, ABState::A)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&ABDebouncer::new(9, ABState::A)),
            4
        );
        assert_eq!(
            std::mem::size_of_val(&ABDebouncer::new(16, ABState::A)),
            4
        );
    }

    /// Ensure that the initial state can be specified.
    #[test]
    fn test_initial_state() {
        let mut debouncer_01 = ABDebouncer::new(2, ABState::A);
        assert_eq!(debouncer_01.update(ABState::A), None);
        assert_eq!(debouncer_01.update(ABState::A), None);
        assert_eq!(debouncer_01.update(ABState::B), None);
        assert_eq!(
            debouncer_01.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );

        let mut debouncer_02 = ABDebouncer::new(2, ABState::A);
        assert_eq!(debouncer_02.update(ABState::B), None);
        assert_eq!(
            debouncer_02.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer_02.update(ABState::A), None);
        assert_eq!(
            debouncer_02.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );

        let mut debouncer_03 = ABDebouncer::new(2, ABState::B);
        assert_eq!(debouncer_03.update(ABState::A), None);
        assert_eq!(
            debouncer_03.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer_03.update(ABState::B), None);
        assert_eq!(
            debouncer_03.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );

        let mut debouncer_04 = ABDebouncer::new(2, ABState::B);
        assert_eq!(debouncer_04.update(ABState::B), None);
        assert_eq!(debouncer_04.update(ABState::B), None);
        assert_eq!(debouncer_04.update(ABState::A), None);
        assert_eq!(
            debouncer_04.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
    }

    // Test sequence (0 = A, 1 = B)
    // 0111100100011110011100010101010111100110010101011100000011110001100001010001101101100111000100000101
    // 0n1nnn0nnnnn1nnn0n1nn0nnnnnnnnnn1nnn0n1n0nnnnnnn1nn0nnnnn1nnn0nn1n0nnnnnnnnn1nnnnnnn0n1nn0nnnnnnnnnn (2)
    // 0nn1nnnnnn0nn1nnnnnnnn0nnnnnnnnnn1nnnnnnnnnnnnnnnnnn0nnnnn1nnn0nnnnnnnnnnnnnnnnnnnnnnnn1nn0nnnnnnnnn (3)
    // 0nnn1nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn0nnnnn1nnnnnnnn0nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn (4)
    // 0nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn (5)

    #[test]
    fn test_long_running_02() {
        let mut debouncer: ABDebouncer = ABDebouncer::new(2, ABState::A);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
    }

    #[test]
    fn test_long_running_04() {
        let mut debouncer: ABDebouncer = ABDebouncer::new(4, ABState::A);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(
            debouncer.update(ABState::B),
            Some(Edge::new(ABState::A, ABState::B))
        );
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(
            debouncer.update(ABState::A),
            Some(Edge::new(ABState::B, ABState::A))
        );
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
        assert_eq!(debouncer.update(ABState::A), None);
        assert_eq!(debouncer.update(ABState::B), None);
    }
}
