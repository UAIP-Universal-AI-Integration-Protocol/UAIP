//! Device capability querying and management

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::models::Device;
use crate::repository::DeviceRepository;
use uaip_core::error::UaipResult;

/// Capability query filters
#[derive(Debug, Clone, Default)]
pub struct CapabilityFilter {
    /// Filter by capability name
    pub capability_name: Option<String>,
    /// Filter by capability type
    pub capability_type: Option<String>,
    /// Filter by supported action
    pub action: Option<String>,
    /// Limit results
    pub limit: Option<i64>,
}

/// Capability summary across all devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySummary {
    /// Total number of devices
    pub total_devices: usize,
    /// Unique capability names
    pub capability_names: HashSet<String>,
    /// Unique capability types
    pub capability_types: HashSet<String>,
    /// Count by capability name
    pub devices_by_capability: HashMap<String, usize>,
    /// Count by capability type
    pub devices_by_type: HashMap<String, usize>,
}

/// Service for capability querying
pub struct CapabilityService {
    repository: DeviceRepository,
}

impl CapabilityService {
    /// Create a new capability service
    pub fn new(repository: DeviceRepository) -> Self {
        Self { repository }
    }

    /// Find devices by capability name
    ///
    /// # Arguments
    /// * `capability_name` - Name of the capability to search for
    ///
    /// # Returns
    /// * `Result<Vec<Device>>` - Devices with the specified capability
    pub async fn find_by_capability_name(&self, capability_name: &str) -> UaipResult<Vec<Device>> {
        use crate::models::DeviceFilter;

        // Get all devices first (in production, this should be a JSONB query)
        let all_devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        // Filter devices that have the specified capability
        let filtered: Vec<Device> = all_devices
            .into_iter()
            .filter(|device| self.device_has_capability(device, capability_name))
            .collect();

        Ok(filtered)
    }

    /// Find devices by capability type
    ///
    /// # Arguments
    /// * `capability_type` - Type of capability (e.g., "sensor", "actuator")
    ///
    /// # Returns
    /// * `Result<Vec<Device>>` - Devices with capabilities of the specified type
    pub async fn find_by_capability_type(&self, capability_type: &str) -> UaipResult<Vec<Device>> {
        use crate::models::DeviceFilter;

        let all_devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        let filtered: Vec<Device> = all_devices
            .into_iter()
            .filter(|device| self.device_has_capability_type(device, capability_type))
            .collect();

        Ok(filtered)
    }

    /// Find devices that support a specific action
    ///
    /// # Arguments
    /// * `action` - Action name (e.g., "read", "write", "toggle")
    ///
    /// # Returns
    /// * `Result<Vec<Device>>` - Devices that support the action
    pub async fn find_by_action(&self, action: &str) -> UaipResult<Vec<Device>> {
        use crate::models::DeviceFilter;

        let all_devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        let filtered: Vec<Device> = all_devices
            .into_iter()
            .filter(|device| self.device_supports_action(device, action))
            .collect();

        Ok(filtered)
    }

    /// Find devices by capability name and action
    ///
    /// # Arguments
    /// * `capability_name` - Name of the capability
    /// * `action` - Action name
    ///
    /// # Returns
    /// * `Result<Vec<Device>>` - Devices matching the criteria
    pub async fn find_by_capability_and_action(
        &self,
        capability_name: &str,
        action: &str,
    ) -> UaipResult<Vec<Device>> {
        use crate::models::DeviceFilter;

        let all_devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        let filtered: Vec<Device> = all_devices
            .into_iter()
            .filter(|device| {
                self.device_has_capability_with_action(device, capability_name, action)
            })
            .collect();

        Ok(filtered)
    }

    /// Get summary of all capabilities across devices
    ///
    /// # Returns
    /// * `Result<CapabilitySummary>` - Summary statistics
    pub async fn get_capability_summary(&self) -> UaipResult<CapabilitySummary> {
        use crate::models::DeviceFilter;

        let all_devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        let mut capability_names = HashSet::new();
        let mut capability_types = HashSet::new();
        let mut devices_by_capability: HashMap<String, usize> = HashMap::new();
        let mut devices_by_type: HashMap<String, usize> = HashMap::new();

        for device in &all_devices {
            if let Ok(capabilities) =
                serde_json::from_value::<Vec<serde_json::Value>>(device.capabilities.clone())
            {
                for cap in capabilities {
                    if let (Some(name), Some(cap_type)) = (
                        cap.get("name").and_then(|v| v.as_str()),
                        cap.get("type").and_then(|v| v.as_str()),
                    ) {
                        capability_names.insert(name.to_string());
                        capability_types.insert(cap_type.to_string());

                        *devices_by_capability.entry(name.to_string()).or_insert(0) += 1;
                        *devices_by_type.entry(cap_type.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        Ok(CapabilitySummary {
            total_devices: all_devices.len(),
            capability_names,
            capability_types,
            devices_by_capability,
            devices_by_type,
        })
    }

    /// Get all unique capability names
    pub async fn get_all_capability_names(&self) -> UaipResult<HashSet<String>> {
        let summary = self.get_capability_summary().await?;
        Ok(summary.capability_names)
    }

    /// Get all unique capability types
    pub async fn get_all_capability_types(&self) -> UaipResult<HashSet<String>> {
        let summary = self.get_capability_summary().await?;
        Ok(summary.capability_types)
    }

    /// Check if device has a specific capability by name
    fn device_has_capability(&self, device: &Device, capability_name: &str) -> bool {
        if let Ok(capabilities) =
            serde_json::from_value::<Vec<serde_json::Value>>(device.capabilities.clone())
        {
            capabilities.iter().any(|cap| {
                cap.get("name")
                    .and_then(|v| v.as_str())
                    .map(|name| name == capability_name)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    /// Check if device has a capability of a specific type
    fn device_has_capability_type(&self, device: &Device, capability_type: &str) -> bool {
        if let Ok(capabilities) =
            serde_json::from_value::<Vec<serde_json::Value>>(device.capabilities.clone())
        {
            capabilities.iter().any(|cap| {
                cap.get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == capability_type)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    /// Check if device supports a specific action
    fn device_supports_action(&self, device: &Device, action: &str) -> bool {
        if let Ok(capabilities) =
            serde_json::from_value::<Vec<serde_json::Value>>(device.capabilities.clone())
        {
            capabilities.iter().any(|cap| {
                cap.get("actions")
                    .and_then(|v| v.as_array())
                    .map(|actions| {
                        actions
                            .iter()
                            .any(|a| a.as_str().map(|s| s == action).unwrap_or(false))
                    })
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    /// Check if device has a capability with a specific action
    fn device_has_capability_with_action(
        &self,
        device: &Device,
        capability_name: &str,
        action: &str,
    ) -> bool {
        if let Ok(capabilities) =
            serde_json::from_value::<Vec<serde_json::Value>>(device.capabilities.clone())
        {
            capabilities.iter().any(|cap| {
                let has_name = cap
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|name| name == capability_name)
                    .unwrap_or(false);

                let has_action = cap
                    .get("actions")
                    .and_then(|v| v.as_array())
                    .map(|actions| {
                        actions
                            .iter()
                            .any(|a| a.as_str().map(|s| s == action).unwrap_or(false))
                    })
                    .unwrap_or(false);

                has_name && has_action
            })
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_filter_default() {
        let filter = CapabilityFilter::default();
        assert!(filter.capability_name.is_none());
        assert!(filter.capability_type.is_none());
        assert!(filter.action.is_none());
    }

    #[test]
    fn test_capability_filter_with_values() {
        let filter = CapabilityFilter {
            capability_name: Some("temperature".to_string()),
            capability_type: Some("sensor".to_string()),
            action: Some("read".to_string()),
            limit: Some(10),
        };

        assert_eq!(filter.capability_name, Some("temperature".to_string()));
        assert_eq!(filter.capability_type, Some("sensor".to_string()));
        assert_eq!(filter.action, Some("read".to_string()));
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_capability_summary() {
        let mut capability_names = HashSet::new();
        capability_names.insert("temperature".to_string());
        capability_names.insert("humidity".to_string());

        let mut capability_types = HashSet::new();
        capability_types.insert("sensor".to_string());

        let mut devices_by_capability = HashMap::new();
        devices_by_capability.insert("temperature".to_string(), 5);
        devices_by_capability.insert("humidity".to_string(), 3);

        let mut devices_by_type = HashMap::new();
        devices_by_type.insert("sensor".to_string(), 8);

        let summary = CapabilitySummary {
            total_devices: 10,
            capability_names,
            capability_types,
            devices_by_capability,
            devices_by_type,
        };

        assert_eq!(summary.total_devices, 10);
        assert_eq!(summary.capability_names.len(), 2);
        assert_eq!(summary.capability_types.len(), 1);
    }
}
