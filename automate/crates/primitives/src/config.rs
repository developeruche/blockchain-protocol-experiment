use alloy::primitives::{Address, Bytes, B256};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub global: GlobalConfig,
    pub automation: Vec<RawAutomationConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalConfig {
    pub rpc_http: String,
    pub rpc_ws: String,
    pub automate_contract: String,
    pub private_key_env: String,
    pub chain_id: u64,
    pub max_fee_per_gas_gwei: u64,
    pub max_priority_fee_gwei: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Deserialize)]
pub struct RawAutomationConfig {
    pub id: String,
    pub trigger: TriggerKind,
    
    // Timeout fields
    pub timeout_ms: Option<u64>,
    
    // Interval fields
    pub interval_ms: Option<u64>,
    
    // EventLog fields
    pub contract_address: Option<String>,
    pub event_signature: Option<String>,
    #[serde(default)]
    pub topic_filters: Vec<RawTopicFilter>,
    
    pub calldata: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    Timeout,
    Interval,
    EventLog,
}

#[derive(Debug, Deserialize)]
pub struct RawTopicFilter {
    pub position: u8,
    pub value: String,
}

// Domain structs
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub global: ValidatedGlobalConfig,
    pub automations: Vec<ValidatedAutomation>,
}

#[derive(Debug, Clone)]
pub struct ValidatedGlobalConfig {
    pub rpc_http: String,
    pub rpc_ws: String,
    pub automate_contract: Address,
    pub private_key_env: String,
    pub chain_id: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee: u128,
    pub gas_limit: u64,
}

#[derive(Debug, Clone)]
pub struct ValidatedAutomation {
    pub id: String,
    pub trigger: TriggerStrategy,
    pub calldata: Bytes,
}

#[derive(Debug, Clone)]
pub enum TriggerStrategy {
    Timeout { duration: Duration },
    Interval { period: Duration },
    EventLog {
        contract: Address,
        event_sig: B256,
        topic_filters: Vec<TopicFilter>,
    },
}

#[derive(Debug, Clone)]
pub struct TopicFilter {
    pub position: u8,
    pub value: B256,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO/Parsing error: {0}")]
    ParseError(String),
    #[error("Invalid global automate_contract address: {0}")]
    InvalidGlobalAddress(String),
    #[error("Automation {id}: Missing field {field}")]
    MissingField { id: String, field: &'static str },
    #[error("Automation {id}: Invalid address {value}")]
    InvalidAddress { id: String, value: String },
    #[error("Automation {id}: Invalid hex in {field}: {error}")]
    InvalidHex { id: String, field: &'static str, error: String },
    #[error("Automation {id}: Duration must be > 0")]
    ZeroDuration { id: String },
    #[error("Automation {id}: Topic filter position must be > 0 and <= 3")]
    InvalidTopicPosition { id: String },
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load(path: &std::path::Path) -> Result<ValidatedConfig, Vec<ConfigError>> {
        let content = std::fs::read_to_string(path).map_err(|e| vec![ConfigError::ParseError(e.to_string())])?;
        let raw: RawConfig = toml::from_str(&content).map_err(|e| vec![ConfigError::ParseError(e.to_string())])?;

        let mut errors = Vec::new();

        let global = match raw.global.automate_contract.parse::<Address>() {
            Ok(addr) => ValidatedGlobalConfig {
                rpc_http: raw.global.rpc_http,
                rpc_ws: raw.global.rpc_ws,
                automate_contract: addr,
                private_key_env: raw.global.private_key_env,
                chain_id: raw.global.chain_id,
                max_fee_per_gas: (raw.global.max_fee_per_gas_gwei as u128) * 1_000_000_000,
                max_priority_fee: (raw.global.max_priority_fee_gwei as u128) * 1_000_000_000,
                gas_limit: raw.global.gas_limit,
            },
            Err(_) => {
                errors.push(ConfigError::InvalidGlobalAddress(raw.global.automate_contract.clone()));
                ValidatedGlobalConfig {
                    rpc_http: raw.global.rpc_http.clone(),
                    rpc_ws: raw.global.rpc_ws.clone(),
                    automate_contract: Address::ZERO,
                    private_key_env: raw.global.private_key_env.clone(),
                    chain_id: raw.global.chain_id,
                    max_fee_per_gas: 0,
                    max_priority_fee: 0,
                    gas_limit: 0,
                }
            }
        };

        let mut automations = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for auto in raw.automation {
            if seen_ids.contains(&auto.id) {
                errors.push(ConfigError::ParseError(format!("Duplicate automation ID: {}", auto.id)));
            }
            seen_ids.insert(auto.id.clone());

            let calldata = match auto.calldata.parse::<Bytes>() {
                Ok(b) => b,
                Err(e) => {
                    errors.push(ConfigError::InvalidHex { id: auto.id.clone(), field: "calldata", error: e.to_string() });
                    Bytes::new()
                }
            };

            let trigger = match auto.trigger {
                TriggerKind::Timeout => {
                    if let Some(ms) = auto.timeout_ms {
                        if ms == 0 {
                            errors.push(ConfigError::ZeroDuration { id: auto.id.clone() });
                            TriggerStrategy::Timeout { duration: Duration::from_millis(1) }
                        } else {
                            TriggerStrategy::Timeout { duration: Duration::from_millis(ms) }
                        }
                    } else {
                        errors.push(ConfigError::MissingField { id: auto.id.clone(), field: "timeout_ms" });
                        TriggerStrategy::Timeout { duration: Duration::from_millis(1) }
                    }
                }
                TriggerKind::Interval => {
                    if let Some(ms) = auto.interval_ms {
                        if ms == 0 {
                            errors.push(ConfigError::ZeroDuration { id: auto.id.clone() });
                            TriggerStrategy::Interval { period: Duration::from_millis(1) }
                        } else {
                            TriggerStrategy::Interval { period: Duration::from_millis(ms) }
                        }
                    } else {
                        errors.push(ConfigError::MissingField { id: auto.id.clone(), field: "interval_ms" });
                        TriggerStrategy::Interval { period: Duration::from_millis(1) }
                    }
                }
                TriggerKind::EventLog => {
                    let contract = match auto.contract_address {
                        Some(ref c) => match c.parse::<Address>() {
                            Ok(addr) => addr,
                            Err(_) => {
                                errors.push(ConfigError::InvalidAddress { id: auto.id.clone(), value: c.clone() });
                                Address::ZERO
                            }
                        },
                        None => {
                            errors.push(ConfigError::MissingField { id: auto.id.clone(), field: "contract_address" });
                            Address::ZERO
                        }
                    };

                    let event_sig = match auto.event_signature {
                        Some(ref s) => alloy::primitives::keccak256(s.as_bytes()),
                        None => {
                            errors.push(ConfigError::MissingField { id: auto.id.clone(), field: "event_signature" });
                            B256::ZERO
                        }
                    };

                    let mut filters = Vec::new();
                    for tf in auto.topic_filters {
                        if tf.position == 0 || tf.position > 3 {
                            errors.push(ConfigError::InvalidTopicPosition { id: auto.id.clone() });
                        }
                        match tf.value.parse::<B256>() {
                            Ok(val) => filters.push(TopicFilter { position: tf.position, value: val }),
                            Err(e) => errors.push(ConfigError::InvalidHex { id: auto.id.clone(), field: "topic_filters.value", error: e.to_string() }),
                        }
                    }

                    TriggerStrategy::EventLog { contract, event_sig, topic_filters: filters }
                }
            };

            automations.push(ValidatedAutomation {
                id: auto.id,
                trigger,
                calldata,
            });
        }

        if errors.is_empty() {
            Ok(ValidatedConfig { global, automations })
        } else {
            Err(errors)
        }
    }
}
