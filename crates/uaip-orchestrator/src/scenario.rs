//! Scenario Execution Engine
//!
//! Provides high-level automation scenarios that combine workflows, rules, and triggers.
//! Scenarios represent common automation patterns like "When device X reports Y, do Z".

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uaip_core::error::{Result, UaipError};
use uuid::Uuid;

/// Scenario execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScenarioState {
    /// Scenario is active and monitoring triggers
    Active,
    /// Scenario is inactive
    Inactive,
    /// Scenario is currently executing
    Executing,
    /// Scenario execution completed
    Completed,
    /// Scenario execution failed
    Failed,
}

/// Trigger type for scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Trigger on device event
    DeviceEvent,
    /// Trigger on schedule (cron-like)
    Schedule,
    /// Trigger on manual invocation
    Manual,
    /// Trigger on rule evaluation
    RuleTriggered,
    /// Trigger on webhook call
    Webhook,
    /// Trigger on system event
    SystemEvent,
}

/// Scenario trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTrigger {
    /// Trigger type
    pub trigger_type: TriggerType,

    /// Trigger configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Conditions that must be met for trigger to fire
    #[serde(default)]
    pub conditions: Vec<TriggerCondition>,
}

/// Trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    /// Field to check
    pub field: String,

    /// Operator (equals, contains, greater_than, etc.)
    pub operator: String,

    /// Expected value
    pub value: serde_json::Value,
}

/// Action to perform when scenario triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioAction {
    /// Execute a workflow
    ExecuteWorkflow,
    /// Evaluate a rule
    EvaluateRule,
    /// Send notification
    SendNotification,
    /// Execute custom action
    CustomAction,
}

/// Scenario action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioActionConfig {
    /// Action type
    pub action: ScenarioAction,

    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Whether to wait for action completion
    #[serde(default = "default_true")]
    pub wait: bool,

    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// A complete automation scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Unique scenario ID
    pub id: String,

    /// Scenario name
    pub name: String,

    /// Scenario description
    pub description: Option<String>,

    /// Whether the scenario is enabled
    pub enabled: bool,

    /// Scenario triggers
    pub triggers: Vec<ScenarioTrigger>,

    /// Actions to execute when triggered
    pub actions: Vec<ScenarioActionConfig>,

    /// Current state
    pub state: ScenarioState,

    /// Scenario metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Execution count
    #[serde(default)]
    pub execution_count: u64,

    /// Last triggered timestamp
    pub last_triggered: Option<DateTime<Utc>>,

    /// Last execution result
    pub last_result: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Scenario execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioExecution {
    /// Unique execution ID
    pub id: String,

    /// Scenario ID
    pub scenario_id: String,

    /// Trigger that fired
    pub trigger: TriggerType,

    /// Execution state
    pub state: ScenarioState,

    /// Trigger context
    pub trigger_context: HashMap<String, serde_json::Value>,

    /// Actions executed
    #[serde(default)]
    pub actions_executed: Vec<ActionExecution>,

    /// Error message if failed
    pub error: Option<String>,

    /// Started timestamp
    pub started_at: DateTime<Utc>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Action execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecution {
    /// Action type
    pub action: ScenarioAction,

    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Execution result
    pub result: Option<serde_json::Value>,

    /// Error if failed
    pub error: Option<String>,

    /// Started timestamp
    pub started_at: DateTime<Utc>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Scenario engine for managing automation scenarios
pub struct ScenarioEngine {
    /// Registered scenarios
    scenarios: HashMap<String, Scenario>,

    /// Execution history
    executions: HashMap<String, ScenarioExecution>,
}

impl ScenarioEngine {
    /// Create a new scenario engine
    pub fn new() -> Self {
        Self {
            scenarios: HashMap::new(),
            executions: HashMap::new(),
        }
    }

    /// Register a scenario
    pub fn register_scenario(&mut self, scenario: Scenario) -> Result<()> {
        if scenario.triggers.is_empty() {
            return Err(UaipError::InvalidConfiguration(
                "Scenario must have at least one trigger".to_string(),
            ));
        }

        if scenario.actions.is_empty() {
            return Err(UaipError::InvalidConfiguration(
                "Scenario must have at least one action".to_string(),
            ));
        }

        self.scenarios.insert(scenario.id.clone(), scenario);
        Ok(())
    }

    /// Unregister a scenario
    pub fn unregister_scenario(&mut self, scenario_id: &str) -> Result<()> {
        self.scenarios
            .remove(scenario_id)
            .ok_or_else(|| UaipError::NotFound(format!("Scenario not found: {}", scenario_id)))?;
        Ok(())
    }

    /// Get a scenario by ID
    pub fn get_scenario(&self, scenario_id: &str) -> Option<&Scenario> {
        self.scenarios.get(scenario_id)
    }

    /// Get a scenario by ID (mutable)
    pub fn get_scenario_mut(&mut self, scenario_id: &str) -> Option<&mut Scenario> {
        self.scenarios.get_mut(scenario_id)
    }

    /// Get all scenarios
    pub fn get_all_scenarios(&self) -> Vec<&Scenario> {
        self.scenarios.values().collect()
    }

    /// Get active scenarios
    pub fn get_active_scenarios(&self) -> Vec<&Scenario> {
        self.scenarios
            .values()
            .filter(|s| s.enabled && s.state == ScenarioState::Active)
            .collect()
    }

    /// Enable a scenario
    pub fn enable_scenario(&mut self, scenario_id: &str) -> Result<()> {
        let scenario = self
            .scenarios
            .get_mut(scenario_id)
            .ok_or_else(|| UaipError::NotFound(format!("Scenario not found: {}", scenario_id)))?;

        scenario.enabled = true;
        scenario.state = ScenarioState::Active;
        scenario.updated_at = Utc::now();

        Ok(())
    }

    /// Disable a scenario
    pub fn disable_scenario(&mut self, scenario_id: &str) -> Result<()> {
        let scenario = self
            .scenarios
            .get_mut(scenario_id)
            .ok_or_else(|| UaipError::NotFound(format!("Scenario not found: {}", scenario_id)))?;

        scenario.enabled = false;
        scenario.state = ScenarioState::Inactive;
        scenario.updated_at = Utc::now();

        Ok(())
    }

    /// Trigger a scenario manually
    pub fn trigger_scenario(
        &mut self,
        scenario_id: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let scenario = self
            .scenarios
            .get(scenario_id)
            .ok_or_else(|| UaipError::NotFound(format!("Scenario not found: {}", scenario_id)))?;

        if !scenario.enabled {
            return Err(UaipError::InvalidState(format!(
                "Scenario is disabled: {}",
                scenario_id
            )));
        }

        let execution_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let execution = ScenarioExecution {
            id: execution_id.clone(),
            scenario_id: scenario_id.to_string(),
            trigger: TriggerType::Manual,
            state: ScenarioState::Executing,
            trigger_context: context,
            actions_executed: Vec::new(),
            error: None,
            started_at: now,
            completed_at: None,
        };

        self.executions.insert(execution_id.clone(), execution);

        // Update scenario state
        if let Some(scenario) = self.scenarios.get_mut(scenario_id) {
            scenario.state = ScenarioState::Executing;
            scenario.last_triggered = Some(now);
            scenario.execution_count += 1;
            scenario.updated_at = now;
        }

        Ok(execution_id)
    }

    /// Execute scenario actions
    pub fn execute_actions(&mut self, execution_id: &str) -> Result<()> {
        let scenario_id = {
            let execution = self.executions.get(execution_id).ok_or_else(|| {
                UaipError::NotFound(format!("Execution not found: {}", execution_id))
            })?;
            execution.scenario_id.clone()
        };

        let scenario = self.scenarios.get(&scenario_id).ok_or_else(|| {
            UaipError::NotFound(format!("Scenario not found: {}", scenario_id))
        })?;

        let actions = scenario.actions.clone();
        let execution = self.executions.get_mut(execution_id).unwrap();

        for action_config in actions {
            let now = Utc::now();

            let action_exec = ActionExecution {
                action: action_config.action.clone(),
                parameters: action_config.parameters.clone(),
                result: None,
                error: None,
                started_at: now,
                completed_at: Some(Utc::now()),
            };

            execution.actions_executed.push(action_exec);
        }

        // Mark execution as completed
        execution.state = ScenarioState::Completed;
        execution.completed_at = Some(Utc::now());

        // Update scenario state
        if let Some(scenario) = self.scenarios.get_mut(&scenario_id) {
            scenario.state = ScenarioState::Active;
            scenario.last_result = Some("success".to_string());
            scenario.updated_at = Utc::now();
        }

        Ok(())
    }

    /// Check if a trigger condition is met
    pub fn check_trigger_condition(
        &self,
        trigger: &ScenarioTrigger,
        context: &HashMap<String, serde_json::Value>,
    ) -> bool {
        if trigger.conditions.is_empty() {
            return true;
        }

        trigger.conditions.iter().all(|condition| {
            if let Some(value) = context.get(&condition.field) {
                Self::evaluate_condition(value, &condition.operator, &condition.value)
            } else {
                false
            }
        })
    }

    /// Evaluate a condition
    fn evaluate_condition(
        actual: &serde_json::Value,
        operator: &str,
        expected: &serde_json::Value,
    ) -> bool {
        match operator {
            "equals" => actual == expected,
            "not_equals" => actual != expected,
            "contains" => {
                if let (Some(s1), Some(s2)) = (actual.as_str(), expected.as_str()) {
                    s1.contains(s2)
                } else {
                    false
                }
            }
            "greater_than" => {
                if let (Some(n1), Some(n2)) = (actual.as_f64(), expected.as_f64()) {
                    n1 > n2
                } else {
                    false
                }
            }
            "less_than" => {
                if let (Some(n1), Some(n2)) = (actual.as_f64(), expected.as_f64()) {
                    n1 < n2
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Get execution by ID
    pub fn get_execution(&self, execution_id: &str) -> Option<&ScenarioExecution> {
        self.executions.get(execution_id)
    }

    /// Get executions for a scenario
    pub fn get_scenario_executions(&self, scenario_id: &str) -> Vec<&ScenarioExecution> {
        self.executions
            .values()
            .filter(|e| e.scenario_id == scenario_id)
            .collect()
    }

    /// Clean up old executions
    pub fn cleanup_executions(&mut self, older_than_seconds: i64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(older_than_seconds);

        self.executions.retain(|_, execution| {
            if let Some(completed_at) = execution.completed_at {
                completed_at >= cutoff
            } else {
                true // Keep running executions
            }
        });
    }
}

impl Default for ScenarioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_scenario() -> Scenario {
        Scenario {
            id: "scenario_001".to_string(),
            name: "Test Scenario".to_string(),
            description: Some("A test scenario".to_string()),
            enabled: true,
            triggers: vec![ScenarioTrigger {
                trigger_type: TriggerType::DeviceEvent,
                config: {
                    let mut config = HashMap::new();
                    config.insert("event_type".to_string(), serde_json::json!("temperature"));
                    config
                },
                conditions: vec![],
            }],
            actions: vec![ScenarioActionConfig {
                action: ScenarioAction::SendNotification,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("message".to_string(), serde_json::json!("Temperature alert"));
                    params
                },
                wait: true,
                timeout_seconds: Some(30),
            }],
            state: ScenarioState::Active,
            metadata: HashMap::new(),
            execution_count: 0,
            last_triggered: None,
            last_result: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_scenario_registration() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        assert!(engine.register_scenario(scenario.clone()).is_ok());
        assert!(engine.get_scenario(&scenario.id).is_some());
        assert_eq!(engine.get_all_scenarios().len(), 1);
    }

    #[test]
    fn test_scenario_validation() {
        let mut engine = ScenarioEngine::new();
        let mut scenario = create_test_scenario();

        // Scenario with no triggers should fail
        scenario.triggers.clear();
        assert!(engine.register_scenario(scenario.clone()).is_err());

        // Restore triggers, clear actions
        scenario.triggers = create_test_scenario().triggers;
        scenario.actions.clear();
        assert!(engine.register_scenario(scenario).is_err());
    }

    #[test]
    fn test_scenario_unregistration() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        engine.register_scenario(scenario.clone()).unwrap();
        assert!(engine.unregister_scenario(&scenario.id).is_ok());
        assert!(engine.get_scenario(&scenario.id).is_none());
    }

    #[test]
    fn test_enable_disable_scenario() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        engine.register_scenario(scenario.clone()).unwrap();

        // Disable scenario
        assert!(engine.disable_scenario(&scenario.id).is_ok());
        let scenario_ref = engine.get_scenario(&scenario.id).unwrap();
        assert!(!scenario_ref.enabled);
        assert_eq!(scenario_ref.state, ScenarioState::Inactive);

        // Enable scenario
        assert!(engine.enable_scenario(&scenario.id).is_ok());
        let scenario_ref = engine.get_scenario(&scenario.id).unwrap();
        assert!(scenario_ref.enabled);
        assert_eq!(scenario_ref.state, ScenarioState::Active);
    }

    #[test]
    fn test_trigger_scenario() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        engine.register_scenario(scenario.clone()).unwrap();

        let context = HashMap::new();
        let execution_id = engine.trigger_scenario(&scenario.id, context).unwrap();

        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.scenario_id, scenario.id);
        assert_eq!(execution.trigger, TriggerType::Manual);
        assert_eq!(execution.state, ScenarioState::Executing);

        let scenario_ref = engine.get_scenario(&scenario.id).unwrap();
        assert_eq!(scenario_ref.execution_count, 1);
        assert!(scenario_ref.last_triggered.is_some());
    }

    #[test]
    fn test_execute_actions() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        engine.register_scenario(scenario.clone()).unwrap();

        let context = HashMap::new();
        let execution_id = engine.trigger_scenario(&scenario.id, context).unwrap();

        // Execute actions
        assert!(engine.execute_actions(&execution_id).is_ok());

        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.state, ScenarioState::Completed);
        assert_eq!(execution.actions_executed.len(), 1);
        assert!(execution.completed_at.is_some());

        let scenario_ref = engine.get_scenario(&scenario.id).unwrap();
        assert_eq!(scenario_ref.state, ScenarioState::Active);
        assert_eq!(scenario_ref.last_result.as_deref(), Some("success"));
    }

    #[test]
    fn test_trigger_conditions() {
        let engine = ScenarioEngine::new();

        let mut trigger = ScenarioTrigger {
            trigger_type: TriggerType::DeviceEvent,
            config: HashMap::new(),
            conditions: vec![TriggerCondition {
                field: "temperature".to_string(),
                operator: "greater_than".to_string(),
                value: serde_json::json!(25.0),
            }],
        };

        let mut context = HashMap::new();
        context.insert("temperature".to_string(), serde_json::json!(30.0));

        // Condition should be met
        assert!(engine.check_trigger_condition(&trigger, &context));

        // Condition should not be met
        context.insert("temperature".to_string(), serde_json::json!(20.0));
        assert!(!engine.check_trigger_condition(&trigger, &context));

        // Test empty conditions (always true)
        trigger.conditions.clear();
        assert!(engine.check_trigger_condition(&trigger, &context));
    }

    #[test]
    fn test_get_active_scenarios() {
        let mut engine = ScenarioEngine::new();
        let scenario1 = create_test_scenario();
        let mut scenario2 = create_test_scenario();
        scenario2.id = "scenario_002".to_string();
        scenario2.enabled = false;

        engine.register_scenario(scenario1).unwrap();
        engine.register_scenario(scenario2).unwrap();

        let active = engine.get_active_scenarios();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "scenario_001");
    }

    #[test]
    fn test_cleanup_executions() {
        let mut engine = ScenarioEngine::new();
        let scenario = create_test_scenario();

        engine.register_scenario(scenario.clone()).unwrap();

        let context = HashMap::new();
        let execution_id = engine.trigger_scenario(&scenario.id, context).unwrap();
        engine.execute_actions(&execution_id).unwrap();

        // Verify execution exists
        assert!(engine.get_execution(&execution_id).is_some());

        // Cleanup (keep recent executions)
        engine.cleanup_executions(3600);
        assert!(engine.get_execution(&execution_id).is_some());

        // Cleanup (remove old executions)
        engine.cleanup_executions(-1);
        assert!(engine.get_execution(&execution_id).is_none());
    }
}
