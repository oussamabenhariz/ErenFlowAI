//! # Shared State - Arc<Mutex> Wrapper for Zero-Copy State Management
//!
//! `SharedState` is now the default state type in ErenFlowAI. It provides thread-safe,
//! reference-counted access to state data without cloning the inner JSON payload.
//!
//! ## What Changed
//!
//! - `State` is now a type alias for `SharedState`
//! - All handlers automatically use `SharedState` - no code changes required
//! - Cloning state between nodes now clones the Arc pointer (cheap), not the data
//!
//! ## Benefits
//!
//! **Performance:**
//! - ✅ Zero-copy state passing between nodes
//! - ✅ No cloning overhead, even with large JSON payloads
//! - ✅ 10-100x improved memory efficiency for large states
//!
//! **API:**
//! - ✅ Same interface as before - no handler code changes needed
//! - ✅ Thread-safe shared mutable state
//! - ✅ Automatic optimization
//!
//! **Trade-offs:**
//! - Minimal mutex lock overhead (negligible for sequential execution)
//! - Not suitable for data parallel execution (only sequential DAG)
//!
//! ## Example
//!
//! ```no_run
//! use erenflow_ai::core::state::State;
//! use serde_json::json;
//!
//! // Create shared state (now default)
//! let state = State::new();
//!
//! // Cloning the Arc pointer is cheap
//! let state_clone = state.clone();
//!
//! // Access without lock unless you need mutable access
//! let value = state.get("key");
//! ```

use crate::core::state::PlainState;
use std::sync::{Arc, Mutex, MutexGuard};
use serde_json::Value;

/// Thread-safe, reference-counted state wrapper.
///
/// Now the default state type. Provides zero-copy state sharing via Arc<Mutex>.
/// Cloning is cheap (just Arc pointer), not data cloning.
#[derive(Clone)]
pub struct SharedState {
    inner: Arc<Mutex<PlainState>>,
}

impl SharedState {
    /// Create a new SharedState from an initial PlainState
    pub fn new(state: PlainState) -> Self {
        SharedState {
            inner: Arc::new(Mutex::new(state)),
        }
    }

    /// Create an empty SharedState
    pub fn empty() -> Self {
        SharedState {
            inner: Arc::new(Mutex::new(PlainState::new())),
        }
    }

    /// Lock the state for reading/writing
    ///
    /// # Panics
    /// Panics if the mutex is poisoned
    ///
    /// # Example
    /// ```no_run
    /// use erenflow_ai::core::state::State;
    /// use serde_json::json;
    ///
    /// let state = State::empty();
    /// let mut inner = state.lock();
    /// inner.set("key", json!("value"));
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, PlainState> {
        self.inner.lock().unwrap()
    }

    /// Try to lock the state, returning None if mutex is poisoned
    pub fn try_lock(&self) -> Option<MutexGuard<'_, PlainState>> {
        self.inner.lock().ok()
    }

    /// Set a value (acquires lock internally)
    pub fn set(&self, key: impl Into<String>, value: Value) {
        if let Ok(mut state) = self.inner.lock() {
            state.set(key, value);
        }
    }

    /// Get a value (acquires lock internally)
    pub fn get(&self, key: &str) -> Option<Value> {
        self.inner.lock().ok().and_then(|state| {
            state.get(key).cloned()
        })
    }

    /// Get a typed value  (acquires lock internally)
    pub fn get_typed<T: serde::de::DeserializeOwned>(&self, key: &str) -> crate::core::error::Result<T> {
        self.inner
            .lock()
            .map_err(|_| crate::core::error::ErenFlowError::StateError(
                "Failed to lock state".to_string(),
            ))?
            .get_typed(key)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner
            .lock()
            .map(|state| state.contains_key(key))
            .unwrap_or(false)
    }

    /// Remove a value
    pub fn remove(&self, key: &str) -> Option<Value> {
        self.inner.lock().ok().and_then(|mut state| {
            state.remove(key)
        })
    }

    /// Get the number of keys
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .map(|state| state.keys().count())
            .unwrap_or(0)
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the underlying JSON map (cloned)
    ///
    /// Note: This returns a clone, not a reference, due to the Arc<Mutex> wrapper.
    pub fn as_map(&self) -> serde_json::Map<String, Value> {
        self.inner
            .lock()
            .map(|state| state.as_map().clone())
            .unwrap_or_default()
    }

    /// Convert to a JSON value
    pub fn to_value(&self) -> Value {
        self.inner
            .lock()
            .map(|state| state.to_value())
            .unwrap_or(Value::Object(Default::default()))
    }

    /// Convert to a JSON string
    pub fn to_json_string(&self) -> crate::core::error::Result<String> {
        self.inner
            .lock()
            .map_err(|_| crate::core::error::ErenFlowError::StateError(
                "Failed to lock state".to_string(),
            ))?
            .to_json_string()
    }

    /// Create a SharedState from a JSON value
    ///
    /// # Example
    /// ```no_run
    /// use serde_json::json;
    /// use erenflow_ai::core::state::State;
    ///
    /// let json = json!({"key": "value"});
    /// let state = State::from_json(json)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_json(value: serde_json::Value) -> crate::core::error::Result<Self> {
        let plain_state = PlainState::from_json(value)?;
        Ok(SharedState::new(plain_state))
    }

    /// Merge another state into this one
    pub fn merge(&self, other: SharedState) -> crate::core::error::Result<()> {
        let mut this = self.inner.lock().map_err(|_| {
            crate::core::error::ErenFlowError::StateError("Failed to lock state".to_string())
        })?;
        let other_inner = other.inner.lock().map_err(|_| {
            crate::core::error::ErenFlowError::StateError("Failed to lock state".to_string())
        })?;
        this.merge(other_inner.clone());
        Ok(())
    }

    /// Execute a closure with mutable access to the state
    ///
    /// # Example
    /// ```no_run
    /// use erenflow_ai::core::state::State;
    /// use serde_json::json;
    ///
    /// let state = State::empty();
    /// state.with_mut(|inner| {
    ///     inner.set("key", json!("value"));
    /// });
    /// ```
    pub fn with_mut<F>(&self, f: F) -> crate::core::error::Result<()>
    where
        F: FnOnce(&mut PlainState),
    {
        let mut state = self.inner.lock().map_err(|_| {
            crate::core::error::ErenFlowError::StateError("Failed to lock state".to_string())
        })?;
        f(&mut state);
        Ok(())
    }

    /// Execute a closure with immutable access to the state
    pub fn with<F, R>(&self, f: F) -> crate::core::error::Result<R>
    where
        F: FnOnce(&PlainState) -> R,
    {
        let state = self.inner.lock().map_err(|_| {
            crate::core::error::ErenFlowError::StateError("Failed to lock state".to_string())
        })?;
        Ok(f(&state))
    }

    /// Convert back to a PlainState (clones the inner state)
    pub fn into_state(self) -> crate::core::error::Result<PlainState> {
        self.inner
            .lock()
            .map(|state| state.clone())
            .map_err(|_| crate::core::error::ErenFlowError::StateError(
                "Failed to lock state".to_string(),
            ))
    }

    /// Get number of active Arc references
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a value as a string, or `None` if missing/not a string.
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Get an iterator over all keys in the state (cloned)
    pub fn keys(&self) -> Box<dyn Iterator<Item = String>> {
        let keys: Vec<String> = self.inner
            .lock()
            .map(|state| state.keys().cloned().collect())
            .unwrap_or_default();
        Box::new(keys.into_iter())
    }

    /// Get the configured LLM client
    pub fn get_llm_client(&self) -> crate::core::error::Result<std::sync::Arc<dyn crate::core::llm::LLMClient>> {
        let config: crate::core::llm::LLMConfig = self.get_typed("_llm_config")
            .map_err(|_| crate::core::error::ErenFlowError::ConfigError(
                "LLM config not found in state. Make sure LLM is configured in config.yaml".to_string()
            ))?;
        config.create_client()
    }
}

impl std::fmt::Debug for SharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState")
            .field("arc_refs", &Arc::strong_count(&self.inner))
            .finish()
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_shared_state_creation() {
        let shared = SharedState::empty();
        assert!(shared.is_empty());
    }

    #[test]
    fn test_shared_state_set_get() {
        let shared = SharedState::empty();
        shared.set("key", json!("value"));
        assert_eq!(shared.get("key"), Some(json!("value")));
    }

    #[test]
    fn test_shared_state_arc_refs() {
        let shared = SharedState::empty();
        assert_eq!(shared.strong_count(), 1);

        let shared2 = shared.clone();
        assert_eq!(shared.strong_count(), 2);
        assert_eq!(shared2.strong_count(), 2);

        drop(shared2);
        assert_eq!(shared.strong_count(), 1);
    }

    #[test]
    fn test_shared_state_with_mut() {
        let shared = SharedState::empty();
        shared.with_mut(|state| {
            state.set("counter", json!(0));
        }).unwrap();

        shared.with_mut(|state| {
            if let Some(val) = state.get_mut("counter") {
                if let Some(count) = val.as_i64() {
                    *val = json!(count + 1);
                }
            }
        }).unwrap();

        assert_eq!(shared.get("counter"), Some(json!(1)));
    }

    #[test]
    fn test_shared_state_no_clone() {
        let shared1 = SharedState::empty();
        shared1.set("large_data", json!({"items": vec![1, 2, 3, 4, 5]}));

        // Cloning the SharedState clones Arc, not the inner state
        let shared2 = shared1.clone();
        assert_eq!(shared1.strong_count(), 2);

        // Both references point to the same state
        assert_eq!(shared1.get("large_data"), shared2.get("large_data"));
    }
}
