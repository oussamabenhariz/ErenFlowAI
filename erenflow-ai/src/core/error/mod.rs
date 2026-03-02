//! # Error Handling
//!
//! Comprehensive error types for all ErenFlowAI operations.
//!
//! This module provides a unified error type `ErenFlowError` that covers all possible
//! failure modes in the framework, along with convenient conversion implementations
//! for common error sources.
//!
//! ## Usage
//!
//! ```no_run
//! use erenflow_ai::core::error::{Result, ErenFlowError};
//!
//! // Most functions return Result<T>
//! fn my_operation() -> Result<String> {
//!     Err(ErenFlowError::ConfigError("Invalid configuration".into()))
//! }
//! ```

use thiserror::Error;

/// Convenient type alias for Results in ErenFlowAI operations
pub type Result<T> = std::result::Result<T, ErenFlowError>;

/// Comprehensive error enum covering all possible ErenFlowAI failures
///
/// Each variant is designed to provide context about what went wrong,
/// making it easy to handle different error scenarios.
#[derive(Error, Debug, Clone)]
pub enum ErenFlowError {
    // Configuration Errors
    /// Invalid or malformed configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    // Graph Errors
    /// Issues with graph structure or operations
    #[error("Graph error: {0}")]
    GraphError(String),

    /// Node not found in the graph
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Invalid edge definition
    #[error("Invalid edge: {0}")]
    InvalidEdge(String),

    /// Cycle detected in acyclic graph
    #[error("Cycle detected in graph")]
    CycleDetected,

    // Node/Runtime Errors
    /// Error during node execution
    #[error("Node execution error: {0}")]
    NodeExecutionError(String),

    /// Runtime orchestration error
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Execution failed
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Execution aborted by middleware
    #[error("Execution aborted: {0}")]
    ExecutionAborted(String),

    /// Operation timed out
    #[error("Timeout error")]
    TimeoutError,

    /// Execution timeout
    #[error("Execution timeout: {0}")]
    ExecutionTimeout(String),

    // Parallel Execution Errors
    /// Error during parallel execution
    #[error("Parallel execution error: {0}")]
    ParallelExecutionError(String),

    // State Errors
    /// Error with state management
    #[error("State error: {0}")]
    StateError(String),

    // Service Integration Errors
    /// LLM operation failed
    #[error("LLM error: {0}")]
    LLMError(String),

    /// MCP (Model Context Protocol) operation failed
    #[error("MCP error: {0}")]
    MCPError(String),

    /// Tool operation failed
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Validation error (e.g., schema validation)
    #[error("Validation error: {0}")]
    ValidationError(String),

    // Serialization Errors
    /// JSON serialization/deserialization failed
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// YAML parsing failed
    #[error("YAML parse error: {0}")]
    YamlError(String),

    // System Errors
    /// File I/O operation failed
    #[error("IO error: {0}")]
    IoError(String),
}

// Convenience conversions for common string types
impl From<String> for ErenFlowError {
    fn from(s: String) -> Self {
        ErenFlowError::ExecutionError(s)
    }
}

impl From<&str> for ErenFlowError {
    fn from(s: &str) -> Self {
        ErenFlowError::ExecutionError(s.to_string())
    }
}
impl From<serde_json::Error> for ErenFlowError {
    fn from(e: serde_json::Error) -> Self {
        ErenFlowError::SerializationError(e.to_string())
    }
}

impl From<std::io::Error> for ErenFlowError {
    fn from(e: std::io::Error) -> Self {
        ErenFlowError::IoError(e.to_string())
    }
}

// =============================================================================
// Error Context & Helper Methods
// =============================================================================

impl ErenFlowError {
    /// Add context to an error message
    ///
    /// Useful for providing debugging information without re-wrapping the error.
    ///
    /// # Example
    /// ```no_run
    /// use erenflow_ai::core::error::ErenFlowError;
    ///
    /// let err = ErenFlowError::ConfigError("invalid value".to_string());
    /// let contextualized = err.context("while loading agent config from 'config.yaml'");
    /// ```
    pub fn context(self, msg: &str) -> Self {
        match self {
            ErenFlowError::ConfigError(inner) => {
                ErenFlowError::ConfigError(format!("{}\nContext: {}", inner, msg))
            }
            ErenFlowError::NodeExecutionError(inner) => {
                ErenFlowError::NodeExecutionError(format!("{}\nContext: {}", inner, msg))
            }
            ErenFlowError::RuntimeError(inner) => {
                ErenFlowError::RuntimeError(format!("{}\nContext: {}", inner, msg))
            }
            ErenFlowError::StateError(inner) => {
                ErenFlowError::StateError(format!("{}\nContext: {}", inner, msg))
            }
            ErenFlowError::LLMError(inner) => {
                ErenFlowError::LLMError(format!("{}\nContext: {}", inner, msg))
            }
            other => other,
        }
    }

    /// Check if error is a timeout
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            ErenFlowError::TimeoutError | ErenFlowError::ExecutionTimeout(_)
        )
    }

    /// Check if error is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, ErenFlowError::ValidationError(_))
    }

    /// Check if error is an LLM error
    pub fn is_llm_error(&self) -> bool {
        matches!(self, ErenFlowError::LLMError(_))
    }

    /// Check if error is a state-related error
    pub fn is_state_error(&self) -> bool {
        matches!(self, ErenFlowError::StateError(_))
    }

    /// Get a hint for common error scenarios
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            ErenFlowError::NodeNotFound(_) => {
                Some("Make sure the handler name in config.yaml matches a #[register_handler] function name")
            }
            ErenFlowError::StateError(msg) if msg.contains("not found") => {
                Some("Check that previous nodes set this state field, or set it in main before agent.run()")
            }
            ErenFlowError::LLMError(msg) if msg.contains("401") || msg.contains("Unauthorized") => {
                Some("Check your API key is valid and set correctly (e.g., $MISTRAL_API_KEY environment variable)")
            }
            ErenFlowError::TimeoutError | ErenFlowError::ExecutionTimeout(_) => {
                Some("Increase the timeout value in config.yaml for this node, or optimize handler performance")
            }
            ErenFlowError::ConfigError(_) => {
                Some("Ensure config.yaml is valid YAML and all required fields are present")
            }
            _ => None,
        }
    }
}
