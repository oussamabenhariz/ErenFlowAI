//! # State Optimization Utilities
//!
//! Provides utilities to optimize state handling and reduce cloning during execution.
//!
//! ## Strategies
//!
//! 1. **Lazy Clone** - Only clone state when mutations occur
//! 2. **Reference Counting** - Use Arc<Mutex> for shared state
//! 3. **Parallel Execution** - Efficient state distribution to parallel nodes

use crate::core::state::State;
use std::sync::Arc;

/// Lazy-clone state wrapper that avoids cloning until actual mutations
///
/// This implements a simple copy-on-write pattern for state.
/// Great for cases where most nodes only read from state.
#[derive(Clone)]
pub struct OptimizedState {
    inner: Arc<State>,
    is_owned: bool,
}

impl OptimizedState {
    /// Create a new optimized state from a base state
    pub fn new(state: State) -> Self {
        OptimizedState {
            inner: Arc::new(state),
            is_owned: true,
        }
    }

    /// Get a reference to the inner state for reading
    pub fn as_ref(&self) -> &State {
        &self.inner
    }

    /// Get mutable access, cloning first if this is a shared reference
    pub fn as_mut(&mut self) -> &mut State {
        if !self.is_owned || Arc::strong_count(&self.inner) > 1 {
            // Clone only if we don't own it exclusively
            let state = self.inner.as_ref().clone();
            self.inner = Arc::new(state);
            self.is_owned = true;
        }
        Arc::get_mut(&mut self.inner).unwrap()
    }

    /// Convert to owned state
    pub fn into_owned(self) -> State {
        Arc::try_unwrap(self.inner)
            .unwrap_or_else(|arc| arc.as_ref().clone())
    }

    /// Check if this is an exclusive owner
    pub fn is_owned(&self) -> bool {
        self.is_owned && Arc::strong_count(&self.inner) == 1
    }

    /// Get number of active references
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

/// Statistics for tracking cloning efficiency
#[derive(Debug, Clone, Default)]
pub struct CloneStats {
    /// Number of state clones performed
    pub clone_count: u64,
    /// Number of state mutations
    pub mutation_count: u64,
    /// Estimated bytes cloned (rough estimation)
    pub estimated_bytes_cloned: u64,
}

impl CloneStats {
    /// Calculate efficiency ratio (mutations / clones)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.clone_count == 0 {
            return 0.0;
        }
        self.mutation_count as f64 / self.clone_count as f64
    }

    /// Format nice summary
    pub fn summary(&self) -> String {
        format!(
            "Clones: {}, Mutations: {}, Efficiency: {:.2}%, Est. Data: {} bytes",
            self.clone_count,
            self.mutation_count,
            self.efficiency_ratio() * 100.0,
            self.estimated_bytes_cloned
        )
    }
}

/// Guidelines for when to use different state strategies
pub mod strategies {
    /// Use standard State (with Clone) when:
    /// - Nodes are few and state updates are frequent
    /// - Small JSON payloads (< 1MB)
    /// - Sequential execution (no parallelism needed)
    /// - Default behavior, always safe
    pub struct StandardStateStrategy;

    /// Use SharedState when:
    /// - State is very large (multiple MB of JSON)
    /// - Many nodes read without modifying
    /// - Checkpointing/resumability is critical
    /// - You accept slight overhead from mutex locks
    pub struct SharedStateStrategy;

    /// Use OptimizedState when:
    /// - Mixed read/write patterns
    /// - Want lazy cloning only on mutation
    /// - Performance-sensitive applications
    pub struct OptimizedStateStrategy;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_optimized_state_no_clone_on_read() {
        let state = State::new();
        let opt = OptimizedState::new(state);

        // Reading doesn't require ownership
        let opt2 = opt.clone();
        assert_eq!(opt.strong_count(), 2);
        assert_eq!(opt2.strong_count(), 2);
    }

    #[test]
    fn test_clone_stats_efficiency() {
        let stats = CloneStats {
            clone_count: 10,
            mutation_count: 8,
            estimated_bytes_cloned: 1024,
        };
        assert!(stats.efficiency_ratio() > 0.7);
        assert!(stats.efficiency_ratio() < 0.9);
    }
}
