use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level WebSocket message exchanged with the VPS backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: String,
}

/// A single managed VPS instance as returned by the SkyModCommands backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// IP address of the host machine running the instance.
    #[serde(rename = "HostMachineIp", default)]
    pub host_machine_ip: String,
    /// User ID of the owner (used for logging and identification).
    #[serde(rename = "OwnerId", default)]
    pub owner_id: String,
    /// Unique ID for this instance.
    #[serde(rename = "Id")]
    pub id: String,
    /// Application kind (e.g. "FBAF", "TPM").
    #[serde(rename = "AppKind", default)]
    pub app_kind: String,
    /// When this instance was created.
    #[serde(rename = "CreatedAt", default)]
    pub created_at: String,
    /// Billing expiry timestamp.  The instance MUST be stopped when this
    /// time has passed. The format is ISO-8601 / RFC-3339.
    #[serde(rename = "PaidUntil", default)]
    pub paid_until: String,
    /// Arbitrary key-value context.  The presence of `"turnedOff"` means the
    /// instance should not be running.
    #[serde(rename = "Context", default)]
    pub context: HashMap<String, String>,
    /// Public IP assigned to the instance (if any).
    #[serde(rename = "PublicIp", default)]
    pub public_ip: String,
}

impl Instance {
    /// Returns `true` if the instance should NOT be running (the `"turnedOff"`
    /// key is present in `context`).
    pub fn is_turned_off(&self) -> bool {
        self.context.contains_key("turnedOff")
    }

    /// Parse `paid_until` into a `chrono::DateTime<Utc>`.
    /// Returns `None` if parsing fails.
    pub fn paid_until_dt(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(&self.paid_until)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    /// Returns `true` if `paidUntil` is in the past.
    pub fn is_expired(&self) -> bool {
        match self.paid_until_dt() {
            Some(dt) => dt < chrono::Utc::now(),
            None => true, // treat unparseable as expired
        }
    }

    /// Quick plausibility check.  Returns an error string if the instance
    /// looks invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Instance has empty Id".into());
        }
        if self.owner_id.is_empty() {
            return Err(format!("Instance {} has empty OwnerId", self.id));
        }
        if self.paid_until.is_empty() {
            return Err(format!("Instance {} has empty PaidUntil", self.id));
        }
        if self.paid_until_dt().is_none() {
            return Err(format!(
                "Instance {} has unparseable PaidUntil: {}",
                self.id, self.paid_until
            ));
        }
        Ok(())
    }
}

/// State update pushed by the backend for a single instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpsStateUpdate {
    /// Opaque config object forwarded to the instance.
    #[serde(rename = "Config", default)]
    pub config: Option<serde_json::Value>,
    /// Instance metadata.
    #[serde(rename = "Instance")]
    pub instance: Instance,
    /// Optional extra configuration string.
    #[serde(rename = "ExtraConfig", default)]
    pub extra_config: Option<String>,
}

/// FBAF default settings — analogous to TPM's `NormalDefault` in VpsSocket.cs.
/// These settings are auto-generated for the managed backend UI when no custom
/// config is provided.
pub const FBAF_DEFAULT_SETTINGS: &str = r#"{
    "igns": [""],
    "discordID": "",
    "webhook": "",
    "webhookFormat": "You bought [``{0}``](https:\/\/sky.coflnet.com\/auction\/{7}) for ``{2}`` (``{1}`` profit) in ``{4}ms``",
    "useCookie": true,
    "autoCookie": "1h",
    "relist": true,
    "delay": 500,
    "percentOfTarget": ["0", "10b", 97],
    "listHours": ["0", "10b", 48],
    "clickDelay": 125,
    "bedSpam": false,
    "blockUselessMessages": true,
    "roundTo": 6,
    "skip": {
        "always": false,
        "minProfit": "25m",
        "profitPercentage": "500",
        "minPrice": "500m",
        "userFinder": true,
        "skins": true
    },
    "doNotRelist": {
        "profitOver": "50m",
        "skinned": true,
        "tags": ["HYPERION"],
        "finders": ["USER", "CraftCost"],
        "stacks": false,
        "pingOnFailedListing": false,
        "drillWithParts": true,
        "expiredAuctions": false,
        "slots": []
    },
    "autoRotate": {}
}"#;
