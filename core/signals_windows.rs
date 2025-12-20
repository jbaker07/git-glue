// core/signals_windows.rs
// Windows-specific signals (deterministic, evidence-backed)
// Reuses core SignalEngine; these are signal types only

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WindowsSignal {
    /// LOLBin abuse: rundll32, regsvr32, mshta, powershell, wscript, certutil, bitsadmin, etc.
    LolbinAbuse {
        binary: String,
        command_hash: String,
    },

    /// Execution from user-writable paths (Downloads, Desktop, Temp, AppData)
    UserWritableExec {
        exe_path: String,
        parent_exe: Option<String>,
    },

    /// Interpreter (powershell, cmd, python) exec from writable location
    InterpreterExecFromWritable {
        interpreter: String,
        location: String,
    },

    /// Service persistence: new service install + binary path suspicious/writable
    SuspiciousServicePersistence {
        service_name: String,
        binary_path: String,
        is_writable: bool,
    },

    /// Scheduled task persistence: new task with suspicious action
    ScheduledTaskPersistence {
        task_name: String,
        action: String,
    },

    /// Registry run key persistence: Run/RunOnce → suspicious target
    RegistryRunKeyPersistence {
        key_path: String,
        value: String,
    },

    /// New remote IP seen for executable (lateral movement hint)
    NewRemoteIpForExe {
        exe: String,
        ip: String,
    },

    /// High connection rate burst (potential C2 or worm)
    HighConnRateBurst {
        pid: i32,
        exe: String,
        conn_count: u32,
        window_secs: u32,
    },

    /// Logon anomaly: many failures followed by success
    LogonAnomalyBurst {
        account: String,
        failure_count: u32,
        success_ts: u64,
    },

    /// Defense evasion attempt: log clearing, security log tamper, Defender disable
    DefenseEvasionAttempt {
        technique: String,
        description: String,
    },

    /// Lateral movement hint: remote service/task creation
    LateralMovementHint {
        source_host: String,
        target_host: String,
        method: String,
    },

    /// WMI execution (visible process creation)
    WmiExecution {
        caller_exe: String,
        spawned_exe: String,
    },

    /// Script execution (PowerShell, VBScript)
    ScriptExecution {
        script_type: String,
        script_hash: String,
        source: Option<String>,
    },

    /// Credential dumping: LSASS access or similar
    CredentialDumpAttempt {
        tool: String,
        target_process: String,
    },

    /// Registry manipulation: suspicious hives or keys
    RegistryManipulation {
        key_path: String,
        action: String,
    },
}

impl WindowsSignal {
    /// Map to MITRE ATT&CK technique
    pub fn technique(&self) -> Option<&'static str> {
        match self {
            Self::LolbinAbuse { .. } => Some("T1218"),
            Self::UserWritableExec { .. } => Some("T1204.002"),
            Self::InterpreterExecFromWritable { .. } => Some("T1059"),
            Self::SuspiciousServicePersistence { .. } => Some("T1543.003"),
            Self::ScheduledTaskPersistence { .. } => Some("T1053.005"),
            Self::RegistryRunKeyPersistence { .. } => Some("T1547.001"),
            Self::NewRemoteIpForExe { .. } => Some("T1570"),
            Self::HighConnRateBurst { .. } => Some("T1071"),
            Self::LogonAnomalyBurst { .. } => Some("T1110"),
            Self::DefenseEvasionAttempt { .. } => Some("T1562"),
            Self::LateralMovementHint { .. } => Some("T1570"),
            Self::WmiExecution { .. } => Some("T1047"),
            Self::ScriptExecution { .. } => Some("T1059.001"),
            Self::CredentialDumpAttempt { .. } => Some("T1003"),
            Self::RegistryManipulation { .. } => Some("T1112"),
        }
    }

    /// Default baseline score (0.0..=1.0) for this signal type
    pub fn baseline_score(&self) -> f32 {
        match self {
            Self::LolbinAbuse { .. } => 0.65,
            Self::UserWritableExec { .. } => 0.50,
            Self::InterpreterExecFromWritable { .. } => 0.60,
            Self::SuspiciousServicePersistence { .. } => 0.70,
            Self::ScheduledTaskPersistence { .. } => 0.65,
            Self::RegistryRunKeyPersistence { .. } => 0.60,
            Self::NewRemoteIpForExe { .. } => 0.55,
            Self::HighConnRateBurst { .. } => 0.70,
            Self::LogonAnomalyBurst { .. } => 0.65,
            Self::DefenseEvasionAttempt { .. } => 0.75,
            Self::LateralMovementHint { .. } => 0.70,
            Self::WmiExecution { .. } => 0.50,
            Self::ScriptExecution { .. } => 0.55,
            Self::CredentialDumpAttempt { .. } => 0.80,
            Self::RegistryManipulation { .. } => 0.45,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lolbin_abuse_technique() {
        let sig = WindowsSignal::LolbinAbuse {
            binary: "powershell.exe".to_string(),
            command_hash: "abc123".to_string(),
        };
        assert_eq!(sig.technique(), Some("T1218"));
    }

    #[test]
    fn test_baseline_scores() {
        let sig = WindowsSignal::DefenseEvasionAttempt {
            technique: "log_clear".to_string(),
            description: "Security log cleared".to_string(),
        };
        assert!(sig.baseline_score() >= 0.75);
    }
}
