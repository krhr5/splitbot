use crate::app::StatusState;
use crate::config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use splitnow::Order;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedStatus {
    order_id: String,
    latest: Option<Order>,
    deposit_address: Option<String>,
    deposit_amount: Option<f64>,
}

pub fn load_last_order() -> Result<Option<StatusState>> {
    let path = config::last_order_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    let persisted: PersistedStatus =
        serde_json::from_slice(&body).with_context(|| format!("parse {}", path.display()))?;

    Ok(Some(StatusState {
        order_id: Some(persisted.order_id),
        latest: persisted.latest,
        last_poll: None,
        error: None,
        deposit_address: persisted.deposit_address,
        deposit_amount: persisted.deposit_amount,
        show_raw: false,
    }))
}

pub fn save_last_order(status: &StatusState) -> Result<()> {
    let Some(order_id) = status.order_id.clone() else {
        return Ok(());
    };
    let path = config::last_order_path()?;
    let persisted = PersistedStatus {
        order_id,
        latest: status.latest.clone(),
        deposit_address: status.deposit_address.clone(),
        deposit_amount: status.deposit_amount,
    };
    let body = serde_json::to_vec_pretty(&persisted)
        .with_context(|| format!("serialize {}", path.display()))?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
