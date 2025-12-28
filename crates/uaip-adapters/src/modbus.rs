//! Modbus Protocol Adapter
//!
//! Provides Modbus TCP/RTU client functionality for industrial IoT devices.
//! Supports reading and writing coils, discrete inputs, holding registers, and input registers.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info};

use uaip_core::{
    error::{Result, UaipError},
};

/// Modbus function codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

impl FunctionCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::ReadCoils),
            0x02 => Some(Self::ReadDiscreteInputs),
            0x03 => Some(Self::ReadHoldingRegisters),
            0x04 => Some(Self::ReadInputRegisters),
            0x05 => Some(Self::WriteSingleCoil),
            0x06 => Some(Self::WriteSingleRegister),
            0x0F => Some(Self::WriteMultipleCoils),
            0x10 => Some(Self::WriteMultipleRegisters),
            _ => None,
        }
    }
}

/// Modbus adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    /// Modbus server address (host:port)
    pub server_address: String,

    /// Unit/Slave ID
    pub unit_id: u8,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Read timeout in seconds
    pub read_timeout: u64,

    /// Write timeout in seconds
    pub write_timeout: u64,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:502".to_string(),
            unit_id: 1,
            connection_timeout: 10,
            read_timeout: 5,
            write_timeout: 5,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Modbus TCP adapter for industrial device communication
pub struct ModbusAdapter {
    config: ModbusConfig,
    transaction_id: std::sync::atomic::AtomicU16,
}

impl ModbusAdapter {
    /// Create a new Modbus adapter
    pub fn new(config: ModbusConfig) -> Result<Self> {
        info!(
            "Modbus adapter created for server: {} (unit_id: {})",
            config.server_address, config.unit_id
        );

        Ok(Self {
            config,
            transaction_id: std::sync::atomic::AtomicU16::new(1),
        })
    }

    /// Get next transaction ID
    fn next_transaction_id(&self) -> u16 {
        self.transaction_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Connect to Modbus server
    async fn connect(&self) -> Result<TcpStream> {
        let addr: SocketAddr = self
            .config
            .server_address
            .parse()
            .map_err(|e| {
                UaipError::InvalidConfiguration(format!("Invalid server address: {}", e))
            })?;

        let stream = timeout(
            Duration::from_secs(self.config.connection_timeout),
            TcpStream::connect(addr),
        )
        .await
        .map_err(|_| UaipError::Timeout("Connection timeout".to_string()))?
        .map_err(|e| UaipError::ConnectionError(format!("Failed to connect: {}", e)))?;

        debug!("Connected to Modbus server: {}", self.config.server_address);
        Ok(stream)
    }

    /// Build Modbus TCP request header (MBAP Header)
    fn build_mbap_header(&self, transaction_id: u16, length: u16) -> Vec<u8> {
        let mut header = Vec::with_capacity(7);
        // Transaction ID (2 bytes)
        header.extend_from_slice(&transaction_id.to_be_bytes());
        // Protocol ID (2 bytes) - always 0 for Modbus TCP
        header.extend_from_slice(&[0x00, 0x00]);
        // Length (2 bytes) - number of following bytes
        header.extend_from_slice(&length.to_be_bytes());
        // Unit ID (1 byte)
        header.push(self.config.unit_id);
        header
    }

    /// Read coils (function code 0x01)
    pub async fn read_coils(&self, address: u16, count: u16) -> Result<Vec<bool>> {
        if count == 0 || count > 2000 {
            return Err(UaipError::InvalidParameter(
                "Count must be between 1 and 2000".to_string(),
            ));
        }

        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::ReadCoils as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&count.to_be_bytes());

        let response = self.send_request(transaction_id, pdu).await?;
        self.parse_coils_response(&response, count)
    }

    /// Read discrete inputs (function code 0x02)
    pub async fn read_discrete_inputs(&self, address: u16, count: u16) -> Result<Vec<bool>> {
        if count == 0 || count > 2000 {
            return Err(UaipError::InvalidParameter(
                "Count must be between 1 and 2000".to_string(),
            ));
        }

        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::ReadDiscreteInputs as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&count.to_be_bytes());

        let response = self.send_request(transaction_id, pdu).await?;
        self.parse_coils_response(&response, count)
    }

    /// Read holding registers (function code 0x03)
    pub async fn read_holding_registers(&self, address: u16, count: u16) -> Result<Vec<u16>> {
        if count == 0 || count > 125 {
            return Err(UaipError::InvalidParameter(
                "Count must be between 1 and 125".to_string(),
            ));
        }

        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::ReadHoldingRegisters as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&count.to_be_bytes());

        let response = self.send_request(transaction_id, pdu).await?;
        self.parse_registers_response(&response, count)
    }

    /// Read input registers (function code 0x04)
    pub async fn read_input_registers(&self, address: u16, count: u16) -> Result<Vec<u16>> {
        if count == 0 || count > 125 {
            return Err(UaipError::InvalidParameter(
                "Count must be between 1 and 125".to_string(),
            ));
        }

        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::ReadInputRegisters as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&count.to_be_bytes());

        let response = self.send_request(transaction_id, pdu).await?;
        self.parse_registers_response(&response, count)
    }

    /// Write single coil (function code 0x05)
    pub async fn write_single_coil(&self, address: u16, value: bool) -> Result<()> {
        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::WriteSingleCoil as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        let coil_value: u16 = if value { 0xFF00 } else { 0x0000 };
        pdu.extend_from_slice(&coil_value.to_be_bytes());

        self.send_request(transaction_id, pdu).await?;
        debug!("Wrote single coil at address {}: {}", address, value);
        Ok(())
    }

    /// Write single register (function code 0x06)
    pub async fn write_single_register(&self, address: u16, value: u16) -> Result<()> {
        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::WriteSingleRegister as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&value.to_be_bytes());

        self.send_request(transaction_id, pdu).await?;
        debug!("Wrote single register at address {}: {}", address, value);
        Ok(())
    }

    /// Write multiple registers (function code 0x10)
    pub async fn write_multiple_registers(&self, address: u16, values: &[u16]) -> Result<()> {
        if values.is_empty() || values.len() > 123 {
            return Err(UaipError::InvalidParameter(
                "Values count must be between 1 and 123".to_string(),
            ));
        }

        let transaction_id = self.next_transaction_id();
        let mut pdu = vec![FunctionCode::WriteMultipleRegisters as u8];
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&(values.len() as u16).to_be_bytes());
        pdu.push((values.len() * 2) as u8); // Byte count

        for value in values {
            pdu.extend_from_slice(&value.to_be_bytes());
        }

        self.send_request(transaction_id, pdu).await?;
        debug!(
            "Wrote {} registers starting at address {}",
            values.len(),
            address
        );
        Ok(())
    }

    /// Send request with retry logic
    async fn send_request(&self, transaction_id: u16, pdu: Vec<u8>) -> Result<Vec<u8>> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {}", attempt);
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }

            match self.execute_request(transaction_id, &pdu).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    error!("Modbus request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            UaipError::MaxRetriesExceeded("Modbus request retries exhausted".to_string())
        }))
    }

    /// Execute a single request
    async fn execute_request(&self, transaction_id: u16, pdu: &[u8]) -> Result<Vec<u8>> {
        let mut stream = self.connect().await?;

        // Build complete request (MBAP header + PDU)
        let length = (pdu.len() + 1) as u16; // +1 for unit ID
        let header = self.build_mbap_header(transaction_id, length);
        let mut request = header;
        request.extend_from_slice(pdu);

        // Send request
        use tokio::io::AsyncWriteExt;
        timeout(
            Duration::from_secs(self.config.write_timeout),
            stream.write_all(&request),
        )
        .await
        .map_err(|_| UaipError::Timeout("Write timeout".to_string()))?
        .map_err(|e| UaipError::ConnectionError(format!("Failed to send request: {}", e)))?;

        // Read response
        use tokio::io::AsyncReadExt;
        let mut response = vec![0u8; 260]; // Max Modbus TCP frame size
        let n = timeout(
            Duration::from_secs(self.config.read_timeout),
            stream.read(&mut response),
        )
        .await
        .map_err(|_| UaipError::Timeout("Read timeout".to_string()))?
        .map_err(|e| UaipError::ConnectionError(format!("Failed to read response: {}", e)))?;

        response.truncate(n);

        // Validate response
        if response.len() < 8 {
            return Err(UaipError::InvalidMessage(
                "Response too short".to_string(),
            ));
        }

        // Check transaction ID
        let resp_transaction_id = u16::from_be_bytes([response[0], response[1]]);
        if resp_transaction_id != transaction_id {
            return Err(UaipError::InvalidMessage(format!(
                "Transaction ID mismatch: expected {}, got {}",
                transaction_id, resp_transaction_id
            )));
        }

        // Extract PDU (skip MBAP header)
        Ok(response[7..].to_vec())
    }

    /// Parse coils/discrete inputs response
    fn parse_coils_response(&self, pdu: &[u8], count: u16) -> Result<Vec<bool>> {
        if pdu.len() < 2 {
            return Err(UaipError::InvalidMessage("Response too short".to_string()));
        }

        let byte_count = pdu[1] as usize;
        if pdu.len() < 2 + byte_count {
            return Err(UaipError::InvalidMessage(
                "Incomplete response".to_string(),
            ));
        }

        let mut coils = Vec::new();
        for i in 0..count {
            let byte_index = (i / 8) as usize;
            let bit_index = i % 8;
            let byte = pdu[2 + byte_index];
            let value = (byte & (1 << bit_index)) != 0;
            coils.push(value);
        }

        Ok(coils)
    }

    /// Parse registers response
    fn parse_registers_response(&self, pdu: &[u8], count: u16) -> Result<Vec<u16>> {
        if pdu.len() < 2 {
            return Err(UaipError::InvalidMessage("Response too short".to_string()));
        }

        let byte_count = pdu[1] as usize;
        if byte_count != (count * 2) as usize {
            return Err(UaipError::InvalidMessage(
                "Byte count mismatch".to_string(),
            ));
        }

        if pdu.len() < 2 + byte_count {
            return Err(UaipError::InvalidMessage(
                "Incomplete response".to_string(),
            ));
        }

        let mut registers = Vec::new();
        for i in 0..count {
            let offset = 2 + (i * 2) as usize;
            let value = u16::from_be_bytes([pdu[offset], pdu[offset + 1]]);
            registers.push(value);
        }

        Ok(registers)
    }

    /// Get the Modbus configuration
    pub fn get_config(&self) -> &ModbusConfig {
        &self.config
    }

    /// Health check - try to read a register
    pub async fn health_check(&self) -> Result<()> {
        // Try to read one holding register at address 0
        self.read_holding_registers(0, 1).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_config_default() {
        let config = ModbusConfig::default();
        assert_eq!(config.server_address, "127.0.0.1:502");
        assert_eq!(config.unit_id, 1);
        assert_eq!(config.connection_timeout, 10);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_function_code_conversion() {
        assert_eq!(
            FunctionCode::from_u8(0x01),
            Some(FunctionCode::ReadCoils)
        );
        assert_eq!(
            FunctionCode::from_u8(0x03),
            Some(FunctionCode::ReadHoldingRegisters)
        );
        assert_eq!(FunctionCode::from_u8(0xFF), None);
    }

    #[test]
    fn test_modbus_adapter_creation() {
        let config = ModbusConfig::default();
        let result = ModbusAdapter::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mbap_header() {
        let config = ModbusConfig::default();
        let adapter = ModbusAdapter::new(config).unwrap();
        let header = adapter.build_mbap_header(1, 6);

        assert_eq!(header.len(), 7);
        assert_eq!(header[0], 0x00); // Transaction ID high
        assert_eq!(header[1], 0x01); // Transaction ID low
        assert_eq!(header[2], 0x00); // Protocol ID high
        assert_eq!(header[3], 0x00); // Protocol ID low
        assert_eq!(header[4], 0x00); // Length high
        assert_eq!(header[5], 0x06); // Length low
        assert_eq!(header[6], 0x01); // Unit ID
    }

    #[test]
    fn test_parse_registers_response() {
        let config = ModbusConfig::default();
        let adapter = ModbusAdapter::new(config).unwrap();

        // Response PDU: function code + byte count + 2 registers (0x1234, 0x5678)
        let pdu = vec![0x03, 0x04, 0x12, 0x34, 0x56, 0x78];
        let result = adapter.parse_registers_response(&pdu, 2);

        assert!(result.is_ok());
        let registers = result.unwrap();
        assert_eq!(registers.len(), 2);
        assert_eq!(registers[0], 0x1234);
        assert_eq!(registers[1], 0x5678);
    }

    #[test]
    fn test_parse_coils_response() {
        let config = ModbusConfig::default();
        let adapter = ModbusAdapter::new(config).unwrap();

        // Response PDU: function code + byte count + data (0b00001101 = bits 0,2,3 set)
        let pdu = vec![0x01, 0x01, 0b00001101];
        let result = adapter.parse_coils_response(&pdu, 8);

        assert!(result.is_ok());
        let coils = result.unwrap();
        assert_eq!(coils.len(), 8);
        assert!(coils[0]); // bit 0
        assert!(!coils[1]); // bit 1
        assert!(coils[2]); // bit 2
        assert!(coils[3]); // bit 3
        assert!(!coils[4]); // bit 4
    }
}
