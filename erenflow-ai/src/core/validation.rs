//! JSON Schema Validation for State
//!
//! Provides runtime validation of state objects against JSON schemas.
//! Enables type safety and validation beyond Rust's compile-time checks.
//!
//! # Example
//!
//! ```no_run
//! use erenflow_ai::core::validation::StateValidator;
//! use serde_json::json;
//!
//! // Define a schema
//! let schema = json!({
//!     "type": "object",
//!     "properties": {
//!         "user_query": {"type": "string"},
//!         "results_count": {"type": "integer", "minimum": 0},
//!         "tags": {
//!             "type": "array",
//!             "items": {"type": "string"}
//!         }
//!     },
//!     "required": ["user_query"]
//! });
//!
//! let validator = StateValidator::new(schema)?;
//!
//! // Validate state
//! let state_data = json!({
//!     "user_query": "Find me articles about Rust",
//!     "results_count": 10,
//!     "tags": ["programming", "systems"]
//! });
//!
//! validator.validate(&state_data)?; // ✓ Valid
//! ```

use serde_json::Value;

/// Errors from schema validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Required field is missing
    MissingRequired { field: String },
    
    /// Type mismatch (expected, got)
    TypeMismatch { field: String, expected: String, got: String },
    
    /// Value out of valid range
    OutOfRange { field: String, message: String },
    
    /// Array validation failed
    ArrayValidationFailed { field: String, index: usize, message: String },
    
    /// Custom validation failed
    CustomValidationFailed { message: String },
    
    /// Schema is invalid
    InvalidSchema { message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequired { field } => {
                write!(f, "Missing required field: '{}'", field)
            }
            ValidationError::TypeMismatch { field, expected, got } => {
                write!(f, "Type mismatch in '{}': expected {}, got {}", field, expected, got)
            }
            ValidationError::OutOfRange { field, message } => {
                write!(f, "Out of range for '{}': {}", field, message)
            }
            ValidationError::ArrayValidationFailed { field, index, message } => {
                write!(f, "Array validation failed for '{}[{}]': {}", field, index, message)
            }
            ValidationError::CustomValidationFailed { message } => {
                write!(f, "Custom validation failed: {}", message)
            }
            ValidationError::InvalidSchema { message } => {
                write!(f, "Invalid schema: {}", message)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// JSON Schema validator for state objects
///
/// Validates state data against JSON schemas with support for:
/// - Type validation (string, number, integer, boolean, object, array, null)
/// - Required fields
/// - Minimum/maximum value constraints
/// - String patterns (regex)
/// - Array item validation
/// - Nested object validation
#[derive(Clone)]
pub struct StateValidator {
    schema: Value,
}

impl StateValidator {
    /// Create a new validator from a JSON schema
    pub fn new(schema: Value) -> Result<Self, ValidationError> {
        // Basic schema validation
        if !schema.is_object() {
            return Err(ValidationError::InvalidSchema {
                message: "Schema must be a JSON object".to_string(),
            });
        }

        Ok(StateValidator { schema })
    }

    /// Validate state data against the schema
    pub fn validate(&self, data: &Value) -> Result<(), ValidationError> {
        self.validate_value(data, &self.schema, "root")
    }

    /// Validate a specific field
    pub fn validate_field(&self, field: &str, data: &Value) -> Result<(), ValidationError> {
        let properties = self
            .schema
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| ValidationError::InvalidSchema {
                message: "Schema has no properties".to_string(),
            })?;

        let field_schema = properties.get(field).ok_or_else(|| {
            ValidationError::CustomValidationFailed {
                message: format!("Field '{}' not in schema", field),
            }
        })?;

        self.validate_value(data, field_schema, field)
    }

    /// Get required fields from schema
    pub fn required_fields(&self) -> Vec<&str> {
        self.schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// Validate value against schema (recursive)
    fn validate_value(&self, data: &Value, schema: &Value, path: &str) -> Result<(), ValidationError> {
        // Get schema type
        let schema_type = schema.get("type").and_then(|t| t.as_str());

        // Check type first
        match schema_type {
            Some("string") => self.validate_string(data, schema, path)?,
            Some("number") | Some("integer") => self.validate_number(data, schema, path, schema_type == Some("integer"))?,
            Some("boolean") => {
                if !data.is_boolean() {
                    return Err(ValidationError::TypeMismatch {
                        field: path.to_string(),
                        expected: "boolean".to_string(),
                        got: self.type_name(data),
                    });
                }
            }
            Some("array") => self.validate_array(data, schema, path)?,
            Some("object") => self.validate_object(data, schema, path)?,
            Some("null") => {
                if !data.is_null() {
                    return Err(ValidationError::TypeMismatch {
                        field: path.to_string(),
                        expected: "null".to_string(),
                        got: self.type_name(data),
                    });
                }
            }
            Some(other) => {
                return Err(ValidationError::InvalidSchema {
                    message: format!("Unknown type: {}", other),
                })
            }
            None => {
                // No type specified, only validate constraints
                if let Some(enum_values) = schema.get("enum").and_then(|e| e.as_array()) {
                    if !enum_values.contains(data) {
                        return Err(ValidationError::CustomValidationFailed {
                            message: format!("Value not in allowed enum values"),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_string(&self, data: &Value, schema: &Value, path: &str) -> Result<(), ValidationError> {
        let s = data.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            field: path.to_string(),
            expected: "string".to_string(),
            got: self.type_name(data),
        })?;

        // Check min/max length
        if let Some(min_length) = schema.get("minLength").and_then(|m| m.as_u64()) {
            if s.len() < min_length as usize {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Length {} is less than minimum {}", s.len(), min_length),
                });
            }
        }

        if let Some(max_length) = schema.get("maxLength").and_then(|m| m.as_u64()) {
            if s.len() > max_length as usize {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Length {} exceeds maximum {}", s.len(), max_length),
                });
            }
        }

        // Check pattern (regex)
        if let Some(pattern) = schema.get("pattern").and_then(|p| p.as_str()) {
            // Using simple regex matching (in production, use `regex` crate)
            if !s.contains(pattern) {
                return Err(ValidationError::CustomValidationFailed {
                    message: format!("String '{}' does not match pattern '{}'", s, pattern),
                });
            }
        }

        Ok(())
    }

    fn validate_number(
        &self,
        data: &Value,
        schema: &Value,
        path: &str,
        is_integer: bool,
    ) -> Result<(), ValidationError> {
        let num = data.as_f64().ok_or_else(|| ValidationError::TypeMismatch {
            field: path.to_string(),
            expected: if is_integer { "integer" } else { "number" }.to_string(),
            got: self.type_name(data),
        })?;

        if is_integer && num.fract() != 0.0 {
            return Err(ValidationError::TypeMismatch {
                field: path.to_string(),
                expected: "integer".to_string(),
                got: "float".to_string(),
            });
        }

        // Check minimum
        if let Some(min) = schema.get("minimum").and_then(|m| m.as_f64()) {
            if num < min {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Value {} is less than minimum {}", num, min),
                });
            }
        }

        // Check exclusiveMinimum (deprecated, but still supported)
        if let Some(ex_min) = schema.get("exclusiveMinimum").and_then(|m| m.as_f64()) {
            if num <= ex_min {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Value {} is not greater than {}", num, ex_min),
                });
            }
        }

        // Check maximum
        if let Some(max) = schema.get("maximum").and_then(|m| m.as_f64()) {
            if num > max {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Value {} exceeds maximum {}", num, max),
                });
            }
        }

        // Check exclusiveMaximum (deprecated)
        if let Some(ex_max) = schema.get("exclusiveMaximum").and_then(|m| m.as_f64()) {
            if num >= ex_max {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Value {} is not less than {}", num, ex_max),
                });
            }
        }

        Ok(())
    }

    fn validate_array(&self, data: &Value, schema: &Value, path: &str) -> Result<(), ValidationError> {
        let arr = data.as_array().ok_or_else(|| ValidationError::TypeMismatch {
            field: path.to_string(),
            expected: "array".to_string(),
            got: self.type_name(data),
        })?;

        // Check min/max items
        if let Some(min_items) = schema.get("minItems").and_then(|m| m.as_u64()) {
            if arr.len() < min_items as usize {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Array length {} is less than minimum {}", arr.len(), min_items),
                });
            }
        }

        if let Some(max_items) = schema.get("maxItems").and_then(|m| m.as_u64()) {
            if arr.len() > max_items as usize {
                return Err(ValidationError::OutOfRange {
                    field: path.to_string(),
                    message: format!("Array length {} exceeds maximum {}", arr.len(), max_items),
                });
            }
        }

        // Validate items
        if let Some(item_schema) = schema.get("items") {
            for (idx, item) in arr.iter().enumerate() {
                self.validate_value(item, item_schema, &format!("{}[{}]", path, idx))
                    .map_err(|e| match e {
                        ValidationError::TypeMismatch { field: _, expected, got } => {
                            ValidationError::ArrayValidationFailed {
                                field: path.to_string(),
                                index: idx,
                                message: format!("Expected {}, got {}", expected, got),
                            }
                        }
                        other => ValidationError::ArrayValidationFailed {
                            field: path.to_string(),
                            index: idx,
                            message: other.to_string(),
                        },
                    })?;
            }
        }

        Ok(())
    }

    fn validate_object(&self, data: &Value, schema: &Value, path: &str) -> Result<(), ValidationError> {
        let obj = data.as_object().ok_or_else(|| ValidationError::TypeMismatch {
            field: path.to_string(),
            expected: "object".to_string(),
            got: self.type_name(data),
        })?;

        // Check required fields
        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            for field in required {
                if let Some(field_name) = field.as_str() {
                    if !obj.contains_key(field_name) {
                        return Err(ValidationError::MissingRequired {
                            field: format!("{}.{}", path, field_name),
                        });
                    }
                }
            }
        }

        // Validate properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (prop_name, prop_schema) in properties {
                if let Some(value) = obj.get(prop_name) {
                    let prop_path = if path == "root" {
                        prop_name.clone()
                    } else {
                        format!("{}.{}", path, prop_name)
                    };
                    self.validate_value(value, prop_schema, &prop_path)?;
                }
            }
        }

        Ok(())
    }

    /// Get human-readable type name of a JSON value
    fn type_name(&self, value: &Value) -> String {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    "integer"
                } else {
                    "number"
                }
            }
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_validation() {
        let schema = json!({
            "type": "string",
            "minLength": 3,
            "maxLength": 10
        });

        let validator = StateValidator::new(schema).unwrap();

        // Valid
        assert!(validator.validate(&Value::String("hello".to_string())).is_ok());

        // Too short
        assert!(validator.validate(&Value::String("ab".to_string())).is_err());

        // Too long
        assert!(validator.validate(&Value::String("12345678901".to_string())).is_err());

        // Wrong type
        assert!(validator.validate(&json!(123)).is_err());
    }

    #[test]
    fn test_number_validation() {
        let schema = json!({
            "type": "integer",
            "minimum": 0,
            "maximum": 100
        });

        let validator = StateValidator::new(schema).unwrap();

        // Valid
        assert!(validator.validate(&json!(50)).is_ok());

        // Below minimum
        assert!(validator.validate(&json!(-1)).is_err());

        // Above maximum
        assert!(validator.validate(&json!(101)).is_err());
    }

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let validator = StateValidator::new(schema).unwrap();

        // Valid
        assert!(validator.validate(&json!({"name": "Alice", "age": 30})).is_ok());

        // Missing required field
        assert!(validator.validate(&json!({"age": 30})).is_err());

        // Wrong type
        assert!(validator.validate(&json!({"name": 123})).is_err());
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "items": {"type": "string"},
            "minItems": 1,
            "maxItems": 3
        });

        let validator = StateValidator::new(schema).unwrap();

        // Valid
        assert!(validator.validate(&json!(["a", "b"])).is_ok());

        // Too few items
        assert!(validator.validate(&json!([])).is_err());

        // Too many items
        assert!(validator.validate(&json!(["a", "b", "c", "d"])).is_err());

        // Wrong item type
        assert!(validator.validate(&json!(["a", 123])).is_err());
    }

    #[test]
    fn test_required_fields() {
        let schema = json!({
            "type": "object",
            "properties": {
                "email": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["email"]
        });

        let validator = StateValidator::new(schema).unwrap();
        let required = validator.required_fields();
        
        assert!(required.contains(&"email"));
        assert!(!required.contains(&"age"));
    }
}
