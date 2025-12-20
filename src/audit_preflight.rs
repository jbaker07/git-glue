/// audit_preflight: Validate macOS audit subsystem is enabled and working
/// 
/// macOS 14+ disables audit by default. This module detects audit status
/// and provides actionable remediation steps for production deployment.

use std::process::{Command, Stdio};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct AuditStatus {
    pub enabled: bool,
    pub os_version: String,
    pub audit_running: bool,
    pub auditpipe_accessible: bool,
    pub audit_control_exists: bool,
    pub error_msg: Option<String>,
}

pub fn check_audit_preflight() -> AuditStatus {
    let mut status = AuditStatus {
        enabled: false,
        os_version: get_os_version(),
        audit_running: false,
        auditpipe_accessible: false,
        audit_control_exists: false,
        error_msg: None,
    };
    
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("AUDIT PREFLIGHT CHECK");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("OS Version: {}", status.os_version);
    eprintln!("");
    
    // Check 1: Is auditd running?
    status.audit_running = is_auditd_running();
    eprintln!("[1/4] auditd service running: {}", 
        if status.audit_running { "✓ YES" } else { "✗ NO" });
    
    // Check 2: Is /dev/auditpipe accessible?
    status.auditpipe_accessible = is_auditpipe_accessible();
    eprintln!("[2/4] /dev/auditpipe accessible: {}", 
        if status.auditpipe_accessible { "✓ YES" } else { "✗ NO" });
    
    // Check 3: Does /etc/security/audit_control exist?
    status.audit_control_exists = std::path::Path::new("/etc/security/audit_control").exists();
    eprintln!("[3/4] /etc/security/audit_control exists: {}", 
        if status.audit_control_exists { "✓ YES" } else { "✗ NO" });
    
    // Check 4: Is audit subsystem enabled? (audit -c check)
    let audit_enabled = is_audit_enabled();
    eprintln!("[4/4] Audit enabled via 'audit -c': {}", 
        if audit_enabled { "✓ YES" } else { "✗ NO" });
    
    eprintln!("");
    
    status.enabled = status.audit_running && status.auditpipe_accessible && audit_enabled;
    
    if !status.enabled {
        status.error_msg = Some(format_remediation_error(&status));
    }
    
    if status.enabled {
        eprintln!("✅ AUDIT SUBSYSTEM OPERATIONAL");
    } else {
        eprintln!("❌ AUDIT SUBSYSTEM NOT OPERATIONAL");
        if let Some(err) = &status.error_msg {
            eprintln!("");
            eprintln!("{}", err);
        }
    }
    eprintln!("");
    
    status
}

fn get_os_version() -> String {
    match Command::new("sw_vers")
        .arg("-productVersion")
        .output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

fn is_auditd_running() -> bool {
    match Command::new("launchctl")
        .arg("list")
        .stdout(Stdio::piped())
        .output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("com.apple.auditd")
        }
        Err(_) => false,
    }
}

fn is_auditpipe_accessible() -> bool {
    std::fs::metadata("/dev/auditpipe").is_ok()
}

fn is_audit_enabled() -> bool {
    match Command::new("audit")
        .arg("-c")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output() {
        Ok(output) => {
            // audit -c outputs something like "Current audit condition: enabled"
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            stdout.contains("enabled") || !stderr.contains("not enabled")
        }
        Err(_) => false,
    }
}

fn format_remediation_error(status: &AuditStatus) -> String {
    let mut msg = String::new();
    msg.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    msg.push_str("REMEDIATION REQUIRED\n");
    msg.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    
    msg.push_str("macOS 14+ disables the audit subsystem by default.\n");
    msg.push_str("Model A requires OpenBSM audit stream to function.\n\n");
    
    msg.push_str("IMMEDIATE STEPS:\n\n");
    
    if !status.audit_control_exists {
        msg.push_str("1. Enable audit configuration:\n");
        msg.push_str("   $ sudo cp /etc/security/audit_control.example /etc/security/audit_control\n\n");
    }
    
    if !status.audit_running {
        msg.push_str("2. Enable and start auditd:\n");
        msg.push_str("   $ sudo launchctl enable system/com.apple.auditd\n");
        msg.push_str("   $ sudo launchctl kickstart -k system/com.apple.auditd\n\n");
    }
    
    msg.push_str("3. Reboot system (required for audit changes to take effect):\n");
    msg.push_str("   $ sudo shutdown -r now\n\n");
    
    msg.push_str("4. After reboot, verify audit is running:\n");
    msg.push_str("   $ launchctl list | grep auditd\n");
    msg.push_str("   $ praudit -l /dev/auditpipe | head -5\n\n");
    
    msg.push_str("5. Optional: Configure audit flags for better visibility.\n");
    msg.push_str("   See production README for audit_control setup.\n\n");
    
    msg.push_str("REFERENCE:\n");
    msg.push_str("  Apple audit(5): man audit\n");
    msg.push_str("  OpenBSM docs: https://www.openbsm.org/\n");
    msg.push_str("  leancrew.com article on macOS audit\n\n");
    
    msg.push_str("NOTE: The audit subsystem is deprecated and will be removed in a future macOS version.\n");
    msg.push_str("Model A will eventually require EndpointSecurity framework as a fallback.\n");
    
    msg
}

pub fn validate_audit_or_exit(status: &AuditStatus) {
    if !status.enabled {
        if let Some(err) = &status.error_msg {
            eprintln!("{}", err);
        }
        std::process::exit(1);
    }
}
