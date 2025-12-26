//! Rule evaluation engine for device automation
//!
//! Provides a JSON-based rule engine for automating device behaviors based on conditions.
//! Rules can trigger actions when specified conditions are met.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uaip_core::{error::Result, error::UaipError};

/// A rule that can be evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: Option<String>,

    /// Whether the rule is enabled
    pub enabled: bool,

    /// Conditions that must be met
    pub conditions: Vec<Condition>,

    /// Actions to execute when conditions are met
    pub actions: Vec<Action>,

    /// How conditions should be combined (all or any)
    pub condition_mode: ConditionMode,

    /// Priority (higher executes first)
    pub priority: i32,

    /// Cooldown period in seconds (prevent rapid re-triggering)
    pub cooldown_seconds: Option<u64>,

    /// Last execution timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_executed: Option<DateTime<Utc>>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// How to combine multiple conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConditionMode {
    /// All conditions must be true (AND)
    All,
    /// At least one condition must be true (OR)
    Any,
}

/// A condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Field to check (e.g., "temperature", "device.status")
    pub field: String,

    /// Operator to apply
    pub operator: Operator,

    /// Value to compare against
    pub value: serde_json::Value,

    /// Device ID filter (optional)
    pub device_id: Option<String>,
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Contains (for strings/arrays)
    Contains,
    /// Does not contain
    NotContains,
    /// Matches regex pattern
    Matches,
    /// In list
    In,
    /// Not in list
    NotIn,
}

/// An action to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action type
    pub action_type: ActionType,

    /// Target device ID (for device actions)
    pub device_id: Option<String>,

    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Send command to device
    SendCommand,
    /// Update device configuration
    UpdateConfig,
    /// Send notification
    SendNotification,
    /// Trigger webhook
    TriggerWebhook,
    /// Log event
    LogEvent,
    /// Execute another rule
    ExecuteRule,
}

/// Rule evaluation context
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current telemetry data
    pub telemetry: HashMap<String, serde_json::Value>,

    /// Device states
    pub device_states: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Current timestamp
    pub timestamp: DateTime<Utc>,
}

impl EvaluationContext {
    /// Create a new evaluation context
    pub fn new() -> Self {
        Self {
            telemetry: HashMap::new(),
            device_states: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add telemetry data
    pub fn with_telemetry(mut self, key: String, value: serde_json::Value) -> Self {
        self.telemetry.insert(key, value);
        self
    }

    /// Add device state
    pub fn with_device_state(
        mut self,
        device_id: String,
        state: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.device_states.insert(device_id, state);
        self
    }

    /// Get a value from the context by field path
    pub fn get_value(&self, field: &str) -> Option<&serde_json::Value> {
        // Support dot notation: "device.temperature" or just "temperature"
        if let Some((prefix, _suffix)) = field.split_once('.') {
            if prefix == "device" {
                // This is a device field, but we need a device_id
                return None;
            }
        }

        self.telemetry.get(field)
    }

    /// Get a value for a specific device
    pub fn get_device_value(&self, device_id: &str, field: &str) -> Option<&serde_json::Value> {
        self.device_states
            .get(device_id)
            .and_then(|state| state.get(field))
    }
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule engine for evaluating and executing rules
pub struct RuleEngine {
    /// Loaded rules
    rules: Vec<Rule>,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the engine
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
        // Sort by priority (highest first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < initial_len
    }

    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Update a rule
    pub fn update_rule(&mut self, rule: Rule) -> Result<()> {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule.id) {
            self.rules[pos] = rule;
            // Re-sort by priority
            self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
            Ok(())
        } else {
            Err(UaipError::NotFound(format!("Rule not found: {}", rule.id)))
        }
    }

    /// Evaluate all enabled rules and return triggered rule IDs
    pub fn evaluate(&mut self, context: &EvaluationContext) -> Vec<String> {
        let mut triggered = Vec::new();
        let now = Utc::now();

        for rule in &mut self.rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if let Some(cooldown) = rule.cooldown_seconds {
                if let Some(last_executed) = rule.last_executed {
                    let elapsed = now.signed_duration_since(last_executed);
                    if elapsed.num_seconds() < cooldown as i64 {
                        continue; // Still in cooldown
                    }
                }
            }

            // Evaluate conditions
            if Self::evaluate_conditions(rule, context) {
                triggered.push(rule.id.clone());
                rule.last_executed = Some(now);
            }
        }

        triggered
    }

    /// Evaluate conditions for a rule
    fn evaluate_conditions(rule: &Rule, context: &EvaluationContext) -> bool {
        if rule.conditions.is_empty() {
            return true; // No conditions means always true
        }

        match rule.condition_mode {
            ConditionMode::All => rule
                .conditions
                .iter()
                .all(|c| Self::evaluate_condition(c, context)),
            ConditionMode::Any => rule
                .conditions
                .iter()
                .any(|c| Self::evaluate_condition(c, context)),
        }
    }

    /// Evaluate a single condition
    fn evaluate_condition(condition: &Condition, context: &EvaluationContext) -> bool {
        // Get the value to compare
        let actual_value = if let Some(device_id) = &condition.device_id {
            context.get_device_value(device_id, &condition.field)
        } else {
            context.get_value(&condition.field)
        };

        let actual_value = match actual_value {
            Some(v) => v,
            None => return false, // Field not found
        };

        // Perform comparison based on operator
        match condition.operator {
            Operator::Equals => actual_value == &condition.value,
            Operator::NotEquals => actual_value != &condition.value,
            Operator::GreaterThan => {
                Self::compare_numbers(actual_value, &condition.value, |a, b| a > b)
            }
            Operator::GreaterThanOrEqual => {
                Self::compare_numbers(actual_value, &condition.value, |a, b| a >= b)
            }
            Operator::LessThan => {
                Self::compare_numbers(actual_value, &condition.value, |a, b| a < b)
            }
            Operator::LessThanOrEqual => {
                Self::compare_numbers(actual_value, &condition.value, |a, b| a <= b)
            }
            Operator::Contains => Self::contains(actual_value, &condition.value),
            Operator::NotContains => !Self::contains(actual_value, &condition.value),
            Operator::Matches => Self::matches_regex(actual_value, &condition.value),
            Operator::In => Self::in_list(actual_value, &condition.value),
            Operator::NotIn => !Self::in_list(actual_value, &condition.value),
        }
    }

    /// Compare numeric values
    fn compare_numbers<F>(a: &serde_json::Value, b: &serde_json::Value, op: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (a.as_f64(), b.as_f64()) {
            (Some(a_num), Some(b_num)) => op(a_num, b_num),
            _ => false,
        }
    }

    /// Check if value contains another value
    fn contains(haystack: &serde_json::Value, needle: &serde_json::Value) -> bool {
        match (haystack, needle) {
            (serde_json::Value::String(s), serde_json::Value::String(n)) => s.contains(n),
            (serde_json::Value::Array(arr), needle) => arr.contains(needle),
            _ => false,
        }
    }

    /// Check if value matches regex
    fn matches_regex(value: &serde_json::Value, pattern: &serde_json::Value) -> bool {
        match (value.as_str(), pattern.as_str()) {
            (Some(s), Some(p)) => {
                // Simple pattern matching (not full regex for security)
                s.contains(p)
            }
            _ => false,
        }
    }

    /// Check if value is in list
    fn in_list(value: &serde_json::Value, list: &serde_json::Value) -> bool {
        if let serde_json::Value::Array(arr) = list {
            arr.contains(value)
        } else {
            false
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = Rule {
            id: "rule_001".to_string(),
            name: "Temperature Alert".to_string(),
            description: Some("Alert when temperature exceeds 25°C".to_string()),
            enabled: true,
            conditions: vec![],
            actions: vec![],
            condition_mode: ConditionMode::All,
            priority: 10,
            cooldown_seconds: Some(60),
            last_executed: None,
            metadata: HashMap::new(),
        };

        assert_eq!(rule.id, "rule_001");
        assert!(rule.enabled);
    }

    #[test]
    fn test_rule_engine_add_remove() {
        let mut engine = RuleEngine::new();

        let rule = Rule {
            id: "rule_001".to_string(),
            name: "Test Rule".to_string(),
            description: None,
            enabled: true,
            conditions: vec![],
            actions: vec![],
            condition_mode: ConditionMode::All,
            priority: 5,
            cooldown_seconds: None,
            last_executed: None,
            metadata: HashMap::new(),
        };

        engine.add_rule(rule.clone());
        assert_eq!(engine.get_all_rules().len(), 1);

        assert!(engine.remove_rule("rule_001"));
        assert_eq!(engine.get_all_rules().len(), 0);
    }

    #[test]
    fn test_condition_evaluation_equals() {
        let condition = Condition {
            field: "temperature".to_string(),
            operator: Operator::Equals,
            value: serde_json::json!(25.0),
            device_id: None,
        };

        let context = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(25.0));

        assert!(RuleEngine::evaluate_condition(&condition, &context));

        let context2 = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(20.0));

        assert!(!RuleEngine::evaluate_condition(&condition, &context2));
    }

    #[test]
    fn test_condition_evaluation_greater_than() {
        let condition = Condition {
            field: "temperature".to_string(),
            operator: Operator::GreaterThan,
            value: serde_json::json!(25.0),
            device_id: None,
        };

        let context = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(30.0));

        assert!(RuleEngine::evaluate_condition(&condition, &context));

        let context2 = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(20.0));

        assert!(!RuleEngine::evaluate_condition(&condition, &context2));
    }

    #[test]
    fn test_condition_mode_all() {
        let mut engine = RuleEngine::new();

        let rule = Rule {
            id: "rule_001".to_string(),
            name: "Multi-condition".to_string(),
            description: None,
            enabled: true,
            conditions: vec![
                Condition {
                    field: "temperature".to_string(),
                    operator: Operator::GreaterThan,
                    value: serde_json::json!(25.0),
                    device_id: None,
                },
                Condition {
                    field: "humidity".to_string(),
                    operator: Operator::LessThan,
                    value: serde_json::json!(50.0),
                    device_id: None,
                },
            ],
            actions: vec![],
            condition_mode: ConditionMode::All,
            priority: 1,
            cooldown_seconds: None,
            last_executed: None,
            metadata: HashMap::new(),
        };

        engine.add_rule(rule);

        // Both conditions true
        let context = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(30.0))
            .with_telemetry("humidity".to_string(), serde_json::json!(40.0));

        let triggered = engine.evaluate(&context);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], "rule_001");

        // Only one condition true
        let context2 = EvaluationContext::new()
            .with_telemetry("temperature".to_string(), serde_json::json!(30.0))
            .with_telemetry("humidity".to_string(), serde_json::json!(60.0));

        let triggered2 = engine.evaluate(&context2);
        assert_eq!(triggered2.len(), 0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut engine = RuleEngine::new();

        let rule1 = Rule {
            id: "rule_001".to_string(),
            name: "Low Priority".to_string(),
            description: None,
            enabled: true,
            conditions: vec![],
            actions: vec![],
            condition_mode: ConditionMode::All,
            priority: 1,
            cooldown_seconds: None,
            last_executed: None,
            metadata: HashMap::new(),
        };

        let rule2 = Rule {
            id: "rule_002".to_string(),
            name: "High Priority".to_string(),
            description: None,
            enabled: true,
            conditions: vec![],
            actions: vec![],
            condition_mode: ConditionMode::All,
            priority: 10,
            cooldown_seconds: None,
            last_executed: None,
            metadata: HashMap::new(),
        };

        engine.add_rule(rule1);
        engine.add_rule(rule2);

        let rules = engine.get_all_rules();
        assert_eq!(rules[0].id, "rule_002"); // Higher priority first
        assert_eq!(rules[1].id, "rule_001");
    }
}
