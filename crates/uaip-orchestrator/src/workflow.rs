//! Workflow Engine for Multi-Step Automation
//!
//! Provides state machine-based workflow execution for complex device automation scenarios.
//! Supports sequential and parallel execution, conditional branching, and error handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uaip_core::error::{Result, UaipError};
use uuid::Uuid;

/// Workflow execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowState {
    /// Workflow is pending execution
    Pending,
    /// Workflow is currently running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow is paused
    Paused,
    /// Workflow was cancelled
    Cancelled,
}

/// Step execution state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepState {
    /// Step is pending execution
    Pending,
    /// Step is currently running
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

/// Type of workflow step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    /// Execute an action
    Action,
    /// Evaluate a condition
    Condition,
    /// Wait for a duration
    Delay,
    /// Execute steps in parallel
    Parallel,
    /// Execute steps in sequence
    Sequential,
    /// Loop over steps
    Loop,
}

/// A workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Unique step ID
    pub id: String,

    /// Step name
    pub name: String,

    /// Step type
    pub step_type: StepType,

    /// Step configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Steps to execute if this is a container step (parallel/sequential/loop)
    #[serde(default)]
    pub children: Vec<WorkflowStep>,

    /// Condition for step execution (optional)
    pub condition: Option<String>,

    /// Maximum retry attempts
    #[serde(default)]
    pub max_retries: u32,

    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// On error behavior: "fail", "skip", "retry"
    #[serde(default = "default_on_error")]
    pub on_error: String,
}

fn default_on_error() -> String {
    "fail".to_string()
}

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID
    pub id: String,

    /// Workflow name
    pub name: String,

    /// Workflow description
    pub description: Option<String>,

    /// Workflow version
    pub version: String,

    /// Whether the workflow is enabled
    pub enabled: bool,

    /// Workflow steps
    pub steps: Vec<WorkflowStep>,

    /// Input parameters schema
    #[serde(default)]
    pub input_schema: HashMap<String, serde_json::Value>,

    /// Output parameters schema
    #[serde(default)]
    pub output_schema: HashMap<String, serde_json::Value>,

    /// Workflow metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Workflow execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Unique execution ID
    pub id: String,

    /// Workflow ID being executed
    pub workflow_id: String,

    /// Current execution state
    pub state: WorkflowState,

    /// Input data
    pub input: HashMap<String, serde_json::Value>,

    /// Output data
    #[serde(default)]
    pub output: HashMap<String, serde_json::Value>,

    /// Execution context (variables)
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,

    /// Step execution history
    #[serde(default)]
    pub step_history: Vec<StepExecution>,

    /// Current step index
    pub current_step_index: usize,

    /// Error message if failed
    pub error: Option<String>,

    /// Started timestamp
    pub started_at: DateTime<Utc>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Step execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    /// Step ID
    pub step_id: String,

    /// Step name
    pub step_name: String,

    /// Execution state
    pub state: StepState,

    /// Attempt number
    pub attempt: u32,

    /// Input data for the step
    pub input: HashMap<String, serde_json::Value>,

    /// Output data from the step
    pub output: Option<HashMap<String, serde_json::Value>>,

    /// Error message if failed
    pub error: Option<String>,

    /// Started timestamp
    pub started_at: DateTime<Utc>,

    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Workflow engine for execution management
pub struct WorkflowEngine {
    /// Registered workflows
    workflows: HashMap<String, Workflow>,

    /// Active executions
    executions: HashMap<String, WorkflowExecution>,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            executions: HashMap::new(),
        }
    }

    /// Register a workflow
    pub fn register_workflow(&mut self, workflow: Workflow) -> Result<()> {
        if !workflow.enabled {
            return Err(UaipError::InvalidState(format!(
                "Cannot register disabled workflow: {}",
                workflow.id
            )));
        }

        self.workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Unregister a workflow
    pub fn unregister_workflow(&mut self, workflow_id: &str) -> Result<()> {
        self.workflows
            .remove(workflow_id)
            .ok_or_else(|| UaipError::NotFound(format!("Workflow not found: {}", workflow_id)))?;
        Ok(())
    }

    /// Get a workflow by ID
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }

    /// Get all registered workflows
    pub fn get_all_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Start a workflow execution
    pub fn start_execution(
        &mut self,
        workflow_id: &str,
        input: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| UaipError::NotFound(format!("Workflow not found: {}", workflow_id)))?;

        if !workflow.enabled {
            return Err(UaipError::InvalidState(format!(
                "Workflow is disabled: {}",
                workflow_id
            )));
        }

        let execution_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let execution = WorkflowExecution {
            id: execution_id.clone(),
            workflow_id: workflow_id.to_string(),
            state: WorkflowState::Running,
            input,
            output: HashMap::new(),
            context: HashMap::new(),
            step_history: Vec::new(),
            current_step_index: 0,
            error: None,
            started_at: now,
            completed_at: None,
            updated_at: now,
        };

        self.executions.insert(execution_id.clone(), execution);
        Ok(execution_id)
    }

    /// Get execution by ID
    pub fn get_execution(&self, execution_id: &str) -> Option<&WorkflowExecution> {
        self.executions.get(execution_id)
    }

    /// Get execution by ID (mutable)
    pub fn get_execution_mut(&mut self, execution_id: &str) -> Option<&mut WorkflowExecution> {
        self.executions.get_mut(execution_id)
    }

    /// Cancel an execution
    pub fn cancel_execution(&mut self, execution_id: &str) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| UaipError::NotFound(format!("Execution not found: {}", execution_id)))?;

        if execution.state != WorkflowState::Running && execution.state != WorkflowState::Paused {
            return Err(UaipError::InvalidState(format!(
                "Cannot cancel execution in state: {:?}",
                execution.state
            )));
        }

        execution.state = WorkflowState::Cancelled;
        execution.completed_at = Some(Utc::now());
        execution.updated_at = Utc::now();

        Ok(())
    }

    /// Pause an execution
    pub fn pause_execution(&mut self, execution_id: &str) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| UaipError::NotFound(format!("Execution not found: {}", execution_id)))?;

        if execution.state != WorkflowState::Running {
            return Err(UaipError::InvalidState(format!(
                "Cannot pause execution in state: {:?}",
                execution.state
            )));
        }

        execution.state = WorkflowState::Paused;
        execution.updated_at = Utc::now();

        Ok(())
    }

    /// Resume a paused execution
    pub fn resume_execution(&mut self, execution_id: &str) -> Result<()> {
        let execution = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| UaipError::NotFound(format!("Execution not found: {}", execution_id)))?;

        if execution.state != WorkflowState::Paused {
            return Err(UaipError::InvalidState(format!(
                "Cannot resume execution in state: {:?}",
                execution.state
            )));
        }

        execution.state = WorkflowState::Running;
        execution.updated_at = Utc::now();

        Ok(())
    }

    /// Execute next step in a workflow
    pub fn execute_next_step(&mut self, execution_id: &str) -> Result<StepState> {
        let workflow_id = {
            let execution = self.executions.get(execution_id).ok_or_else(|| {
                UaipError::NotFound(format!("Execution not found: {}", execution_id))
            })?;

            if execution.state != WorkflowState::Running {
                return Err(UaipError::InvalidState(format!(
                    "Execution is not running: {:?}",
                    execution.state
                )));
            }

            execution.workflow_id.clone()
        };

        let workflow = self
            .workflows
            .get(&workflow_id)
            .ok_or_else(|| UaipError::NotFound(format!("Workflow not found: {}", workflow_id)))?;

        let execution = self.executions.get_mut(execution_id).unwrap();

        if execution.current_step_index >= workflow.steps.len() {
            // All steps completed
            execution.state = WorkflowState::Completed;
            execution.completed_at = Some(Utc::now());
            execution.updated_at = Utc::now();
            return Ok(StepState::Completed);
        }

        let step = &workflow.steps[execution.current_step_index];
        let step_result = Self::execute_step(step, execution)?;

        // Record step execution
        let step_exec = StepExecution {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            state: step_result.clone(),
            attempt: 1,
            input: execution.context.clone(),
            output: None,
            error: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        execution.step_history.push(step_exec);
        execution.current_step_index += 1;
        execution.updated_at = Utc::now();

        // Check if all steps are completed
        if execution.current_step_index >= workflow.steps.len() {
            execution.state = WorkflowState::Completed;
            execution.completed_at = Some(Utc::now());
        }

        Ok(step_result)
    }

    /// Execute a single step
    fn execute_step(step: &WorkflowStep, execution: &mut WorkflowExecution) -> Result<StepState> {
        // Check condition if present
        if let Some(condition) = &step.condition {
            if !Self::evaluate_condition(condition, &execution.context)? {
                return Ok(StepState::Skipped);
            }
        }

        match step.step_type {
            StepType::Action => {
                // Execute action step
                Self::execute_action_step(step, execution)
            }
            StepType::Condition => {
                // Evaluate condition step
                Self::execute_condition_step(step, execution)
            }
            StepType::Delay => {
                // Delay step (would need async support in real implementation)
                Ok(StepState::Completed)
            }
            StepType::Parallel => {
                // Execute child steps in parallel (simplified for sync implementation)
                Self::execute_parallel_step(step, execution)
            }
            StepType::Sequential => {
                // Execute child steps sequentially
                Self::execute_sequential_step(step, execution)
            }
            StepType::Loop => {
                // Execute child steps in a loop
                Self::execute_loop_step(step, execution)
            }
        }
    }

    /// Execute an action step
    fn execute_action_step(
        step: &WorkflowStep,
        execution: &mut WorkflowExecution,
    ) -> Result<StepState> {
        // Extract action parameters from config
        if let Some(action_type) = step.config.get("action_type") {
            execution
                .context
                .insert("last_action".to_string(), action_type.clone());
        }

        Ok(StepState::Completed)
    }

    /// Execute a condition step
    fn execute_condition_step(
        step: &WorkflowStep,
        execution: &WorkflowExecution,
    ) -> Result<StepState> {
        if let Some(condition_expr) = step.config.get("expression") {
            if let Some(expr_str) = condition_expr.as_str() {
                let result = Self::evaluate_condition(expr_str, &execution.context)?;
                return Ok(if result {
                    StepState::Completed
                } else {
                    StepState::Failed
                });
            }
        }

        Ok(StepState::Completed)
    }

    /// Execute a parallel step (simplified)
    fn execute_parallel_step(
        step: &WorkflowStep,
        execution: &mut WorkflowExecution,
    ) -> Result<StepState> {
        let mut all_completed = true;

        for child_step in &step.children {
            let result = Self::execute_step(child_step, execution)?;
            if result != StepState::Completed {
                all_completed = false;
                if step.on_error == "fail" {
                    return Ok(StepState::Failed);
                }
            }
        }

        Ok(if all_completed {
            StepState::Completed
        } else {
            StepState::Failed
        })
    }

    /// Execute a sequential step
    fn execute_sequential_step(
        step: &WorkflowStep,
        execution: &mut WorkflowExecution,
    ) -> Result<StepState> {
        for child_step in &step.children {
            let result = Self::execute_step(child_step, execution)?;
            if result != StepState::Completed && step.on_error == "fail" {
                return Ok(StepState::Failed);
            }
        }

        Ok(StepState::Completed)
    }

    /// Execute a loop step
    fn execute_loop_step(
        step: &WorkflowStep,
        execution: &mut WorkflowExecution,
    ) -> Result<StepState> {
        let max_iterations = step
            .config
            .get("max_iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        for _i in 0..max_iterations {
            for child_step in &step.children {
                let result = Self::execute_step(child_step, execution)?;
                if result != StepState::Completed && step.on_error == "fail" {
                    return Ok(StepState::Failed);
                }
            }
        }

        Ok(StepState::Completed)
    }

    /// Evaluate a condition expression (simplified)
    fn evaluate_condition(
        condition: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Simple condition evaluation - in production would use a proper expression parser
        if condition == "true" {
            return Ok(true);
        }
        if condition == "false" {
            return Ok(false);
        }

        // Check if variable exists and is truthy
        if let Some(value) = context.get(condition) {
            return Ok(value.as_bool().unwrap_or(false));
        }

        Ok(false)
    }

    /// Get all active executions
    pub fn get_active_executions(&self) -> Vec<&WorkflowExecution> {
        self.executions
            .values()
            .filter(|e| e.state == WorkflowState::Running || e.state == WorkflowState::Paused)
            .collect()
    }

    /// Clean up completed executions older than specified seconds
    pub fn cleanup_executions(&mut self, older_than_seconds: i64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(older_than_seconds);

        self.executions.retain(|_, execution| {
            if let Some(completed_at) = execution.completed_at {
                completed_at >= cutoff
            } else {
                true // Keep running/paused executions
            }
        });
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: "workflow_001".to_string(),
            name: "Test Workflow".to_string(),
            description: Some("A test workflow".to_string()),
            version: "1.0.0".to_string(),
            enabled: true,
            steps: vec![
                WorkflowStep {
                    id: "step_1".to_string(),
                    name: "First Step".to_string(),
                    step_type: StepType::Action,
                    config: {
                        let mut config = HashMap::new();
                        config.insert("action_type".to_string(), serde_json::json!("send_command"));
                        config
                    },
                    children: vec![],
                    condition: None,
                    max_retries: 3,
                    timeout_seconds: Some(30),
                    on_error: "fail".to_string(),
                },
                WorkflowStep {
                    id: "step_2".to_string(),
                    name: "Second Step".to_string(),
                    step_type: StepType::Action,
                    config: HashMap::new(),
                    children: vec![],
                    condition: None,
                    max_retries: 0,
                    timeout_seconds: None,
                    on_error: "fail".to_string(),
                },
            ],
            input_schema: HashMap::new(),
            output_schema: HashMap::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_workflow_registration() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        assert!(engine.register_workflow(workflow.clone()).is_ok());
        assert!(engine.get_workflow(&workflow.id).is_some());
        assert_eq!(engine.get_all_workflows().len(), 1);
    }

    #[test]
    fn test_workflow_unregistration() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();
        assert!(engine.unregister_workflow(&workflow.id).is_ok());
        assert!(engine.get_workflow(&workflow.id).is_none());
    }

    #[test]
    fn test_start_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.workflow_id, workflow.id);
        assert_eq!(execution.state, WorkflowState::Running);
        assert_eq!(execution.current_step_index, 0);
    }

    #[test]
    fn test_execute_steps() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        // Execute first step
        let result = engine.execute_next_step(&execution_id).unwrap();
        assert_eq!(result, StepState::Completed);

        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.current_step_index, 1);
        assert_eq!(execution.step_history.len(), 1);

        // Execute second step
        let result = engine.execute_next_step(&execution_id).unwrap();
        assert_eq!(result, StepState::Completed);

        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.current_step_index, 2);
        assert_eq!(execution.step_history.len(), 2);
        assert_eq!(execution.state, WorkflowState::Completed);
    }

    #[test]
    fn test_pause_resume_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        // Pause execution
        assert!(engine.pause_execution(&execution_id).is_ok());
        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.state, WorkflowState::Paused);

        // Resume execution
        assert!(engine.resume_execution(&execution_id).is_ok());
        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.state, WorkflowState::Running);
    }

    #[test]
    fn test_cancel_execution() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        // Cancel execution
        assert!(engine.cancel_execution(&execution_id).is_ok());
        let execution = engine.get_execution(&execution_id).unwrap();
        assert_eq!(execution.state, WorkflowState::Cancelled);
        assert!(execution.completed_at.is_some());
    }

    #[test]
    fn test_conditional_step() {
        let mut engine = WorkflowEngine::new();

        let mut workflow = create_test_workflow();
        workflow.steps[0].condition = Some("false".to_string());

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        // Execute first step (should be skipped due to condition)
        let result = engine.execute_next_step(&execution_id).unwrap();
        assert_eq!(result, StepState::Skipped);
    }

    #[test]
    fn test_cleanup_executions() {
        let mut engine = WorkflowEngine::new();
        let workflow = create_test_workflow();

        engine.register_workflow(workflow.clone()).unwrap();

        let input = HashMap::new();
        let execution_id = engine.start_execution(&workflow.id, input).unwrap();

        // Complete the workflow
        engine.execute_next_step(&execution_id).unwrap();
        engine.execute_next_step(&execution_id).unwrap();

        // Verify execution exists
        assert!(engine.get_execution(&execution_id).is_some());

        // Cleanup (keep recent executions)
        engine.cleanup_executions(3600);
        assert!(engine.get_execution(&execution_id).is_some());

        // Cleanup (remove old executions - negative value means remove everything completed before now+1s)
        engine.cleanup_executions(-1);
        // Completed execution should be removed
        assert!(engine.get_execution(&execution_id).is_none());
    }
}
