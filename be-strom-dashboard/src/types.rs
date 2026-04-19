use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qty {
    pub value: f64,
    pub unit: String,
}

impl Display for Qty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3} {}", self.value, self.unit)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    pub device: String,
    pub voltage: Qty,
    pub current_1: Qty,
    pub current_2: Qty,
    pub current_3: Qty,
    pub power_1: Qty,
    pub power_2: Qty,
    pub power_3: Qty,
}
