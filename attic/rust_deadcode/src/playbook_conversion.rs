//! Playbook Conversion Matrix: Linux → macOS via Technique Mapping
//!
//! This module documents the systematic conversion of Linux eBPF-based playbooks
//! to macOS OpenBSM-based playbooks using MITRE ATT&CK technique families.
//!
//! TECHNIQUE FAMILIES:
//! 1. Execution (T1059, T1059.004, T1203, etc.)
//! 2. Persistence (T1547, T1547.013, T1547.006, T1037, T1547.001, etc.)
//! 3. Privilege Escalation (T1548, T1548.003, T1548.004, etc.)
//! 4. Credential Access (T1110, T1111, T1187, etc.)
//! 5. Discovery/Collection (T1087, T1010, T1217, T1113, etc.)
//! 6. Exfiltration/C2 (T1041, T1071, T1020, T1048, T1041, etc.)
//!
//! COVERAGE MATRIX:
//! ================================================================================
//! Linux Playbook ID                    | Family         | macOS Variant        | Status
//! ================================================================================
//! pb_outage_ransomware_linux_memfd_inplace  | Execution      | pb_macos_ransomware_proc_crytowrite | Ground-Truth
//! pb_encryptor_linux_memfd_rwx_lowrename    | Execution      | pb_macos_ransomware_memfd_equiv    | Candidate*
//! pb_lotl_cred                              | Execution      | pb_macos_lotl_credential_chain     | Ground-Truth
//! pb_lotl_linux_curl_devshm_systemdrun_memfd| Execution      | pb_macos_lotl_curl_sh_chain        | Ground-Truth
//! pb_cloud_cli_exfil_linux_aws_rclone       | Exfil/C2       | pb_macos_cloud_cli_exfil           | Ground-Truth
//! pb_lateral_linux_ssh_portforward_pivot    | Execution      | pb_macos_lateral_ssh_exec          | Ground-Truth
//! pb_container_breakout_precursors_linux    | Execution      | pb_macos_container_breakout_equiv  | Candidate*
//! pb_crypto_miner_linux_pool_cpu_gpu        | Execution      | pb_macos_crypto_miner_pool         | Ground-Truth
//! pb_exfil_linux_staging_tmpfs_ssh          | Exfil/C2       | pb_macos_exfil_staging_ssh         | Candidate*
//! pb_memfd_unsigned_egress                  | Execution      | pb_macos_unsigned_exec_egress      | Ground-Truth
//! pb_outage_edr_tamper_disable              | Persistence    | pb_macos_edr_tamper_disable        | Ground-Truth
//! pb_outage_ransomware_linux_fileburst_crypto| Execution      | pb_macos_ransomware_filewrite_burst| Ground-Truth
//! ================================================================================
//!
//! * Candidate = Not full ground-truth on macOS (missing network visibility or rich syscalls)
//!               Still emit evidence-first explanations but with lower confidence.
//!
//! MACOS-NATIVE ADDITIONS (not from Linux conversions):
//! ================================================================================
//! ID                                  | Family      | Description
//! ================================================================================
//! pb_macos_launchd_persistence        | Persistence | plist write → launchctl bootstrap → execution chain
//! pb_macos_dyld_injection             | Execution   | DYLD_* env set → unsigned binary exec
//! pb_macos_keychain_access            | CredAccess  | security tool usage + suspicious ancestry
//! pb_macos_lolbin_stager              | Execution   | curl|python|sh chains with suspicious CWD
//! pb_macos_quarantine_unsigned_first  | Execution   | quarantine xattr + unsigned + first-seen + follow-on
//! pb_macos_code_signature_bypass      | Execution   | code signature modification + execution
//! ================================================================================
//!
//! KEY TRANSFORMATIONS:
//!
//! Linux Signal                      → macOS OpenBSM Equivalent
//! ────────────────────────────────────────────────────────────
//! RWX_MAP (mmap RWX)               → ProcExec with suspicious ancestry + unsigned
//! MEMFD_EXEC                        → FileCreate (/dev/fd/...) + ProcExec chain (rare on macOS)
//! Syscall instrumentation          → OpenBSM audit events (limited to BSM_* events)
//! Mass file writes without rename  → FileWrite burst aggregation over time window
//! /proc traversal                  → Not directly observable; use first-seen + rare path heuristics
//! systemd unit creation            → Not applicable; use launchd plist writes
//! eBPF memory detection            → Not available; rely on code signature + entropy + unsigned
//! Network egress cadence           → lsof/netstat enrichment snapshot (periodic fact injection)
//! DNS tunneling                    → Requires NetworkExtension (future); mark candidate-only
//!
//! MACOS FACT TYPES (from telemetry.rs → TelemetryRecord):
//! ────────────────────────────────────────────────────────
//! - ProcExec { exe, cmdline, ppid, uid, cwd }
//! - FileWrite { path, size, ts }
//! - FileCreate { path, ts }
//! - FileRename { old_path, new_path, ts }
//! - FileUnlink { path, ts }
//! - AuthEvent { uid, auth_type, success, ts }
//! - NetConnect { dest_ip, dest_port, protocol, ts } [limited]
//! - LaunchCtlActivity { action, plist_path, ts }
//!
//! Each fact carries an EvidencePtr { segment_id, record_index, ts, event_type }

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybookConversionEntry {
    pub linux_id: String,
    pub technique_family: String, // "Execution", "Persistence", etc.
    pub macos_variant_id: String,
    pub macos_variant_name: String,
    pub slot_mapping: Vec<String>, // Required slots for macOS variant
    pub required_facts: Vec<String>, // ProcExec, FileWrite, etc.
    pub status: String, // "Ground-Truth" or "Candidate"
    pub notes: String,
    // NEW: Truth labels
    pub macos_observable: bool,  // Can be detected with macOS OpenBSM alone?
    pub required_sources: Vec<String>, // ["openbsm", "codesign", "quarantine_xattr", "lsof_snapshot"]
    pub classification: String, // "ground_truth" | "candidate"
}

// Coverage matrix (for documentation)
pub fn get_coverage_matrix() -> Vec<PlaybookConversionEntry> {
    vec![
        PlaybookConversionEntry {
            linux_id: "pb_outage_ransomware_linux_memfd_inplace".to_string(),
            technique_family: "Execution (T1059, T1203)".to_string(),
            macos_variant_id: "pb_macos_ransomware_proc_cryptowrite".to_string(),
            macos_variant_name: "Ransomware: Process + Crypto Write Burst".to_string(),
            slot_mapping: vec![
                "suspicious_process_ancestry".to_string(),
                "file_write_burst".to_string(),
                "no_rename_ratio".to_string(),
            ],
            required_facts: vec![
                "ProcExec (unsigned/rare binary)".to_string(),
                "FileWrite (high volume, low rename ratio)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: Replace RWX_MAP with unsigned binary + entropy analysis".to_string(),
            // Truth labels
            macos_observable: true,
            required_sources: vec!["openbsm".to_string(), "codesign".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_lotl_cred".to_string(),
            technique_family: "Execution (T1059.004 - bash script)".to_string(),
            macos_variant_id: "pb_macos_lotl_credential_chain".to_string(),
            macos_variant_name: "LOTL: Credential Access via Script".to_string(),
            slot_mapping: vec![
                "launchctl_or_bash_exec".to_string(),
                "credential_tool_invocation".to_string(),
            ],
            required_facts: vec![
                "ProcExec (bash, python, ruby, etc.)".to_string(),
                "ProcExec (security, dscl, osascript)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: security command for keychain, dscl for user queries".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_lotl_linux_curl_devshm_systemdrun_memfd".to_string(),
            technique_family: "Execution (T1059 - shell script piping)".to_string(),
            macos_variant_id: "pb_macos_lotl_curl_sh_chain".to_string(),
            macos_variant_name: "LOTL: curl | sh (Execution)".to_string(),
            slot_mapping: vec![
                "curl_execution".to_string(),
                "shell_command_piping".to_string(),
                "suspicious_download_cwd".to_string(),
            ],
            required_facts: vec![
                "ProcExec (curl with URL)".to_string(),
                "ProcExec (sh/bash as child of curl|network-touching parent)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: Standard LOLBIN, very visible in OpenBSM".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_cloud_cli_exfil_linux_aws_rclone".to_string(),
            technique_family: "Exfiltration (T1048, T1020)".to_string(),
            macos_variant_id: "pb_macos_cloud_cli_exfil".to_string(),
            macos_variant_name: "Cloud CLI Exfiltration (aws-cli, rclone)".to_string(),
            slot_mapping: vec![
                "cloud_cli_binary_exec".to_string(),
                "suspicious_ancestry".to_string(),
            ],
            required_facts: vec![
                "ProcExec (aws, rclone, gsutil, etc.)".to_string(),
                "ProcExec (from unusual parent: shell, cron, LaunchAgent)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: No egress visibility without lsof enrichment; using exec chain as proxy".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_lateral_linux_ssh_portforward_pivot".to_string(),
            technique_family: "Lateral Movement (T1021.004 - SSH)".to_string(),
            macos_variant_id: "pb_macos_lateral_ssh_exec".to_string(),
            macos_variant_name: "Lateral Movement: SSH Port Forward".to_string(),
            slot_mapping: vec![
                "ssh_execution_with_args".to_string(),
                "port_forward_args".to_string(),
            ],
            required_facts: vec![
                "ProcExec (ssh with -L, -R, or -D args)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: SSH cmdline parsing; OpenBSM shows full command".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_crypto_miner_linux_pool_cpu_gpu".to_string(),
            technique_family: "Resource Hijacking (T1496)".to_string(),
            macos_variant_id: "pb_macos_crypto_miner_pool".to_string(),
            macos_variant_name: "Crypto Miner Pool Connection".to_string(),
            slot_mapping: vec![
                "crypto_miner_binary".to_string(),
                "known_pool_config".to_string(),
            ],
            required_facts: vec![
                "ProcExec (xmrig, monero-miner, cpuminer, etc.)".to_string(),
                "ProcExec (from suspicious parent or cwd)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: Binary name + suspicious parent + CWD is strong signal".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_outage_ransomware_linux_fileburst_crypto".to_string(),
            technique_family: "Impact (T1486 - file encryption)".to_string(),
            macos_variant_id: "pb_macos_ransomware_filewrite_burst".to_string(),
            macos_variant_name: "Ransomware: File Write Burst + Encryption".to_string(),
            slot_mapping: vec![
                "suspicious_process_or_ancestry".to_string(),
                "file_write_burst_ratio".to_string(),
                "entropy_drop_detection".to_string(),
            ],
            required_facts: vec![
                "ProcExec (unsigned, rare, or suspicious)".to_string(),
                "FileWrite (high count, low rename ratio)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: Aggregated file write detection; entropy via binary analysis".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string()],
            classification: "ground_truth".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_exfil_linux_staging_tmpfs_ssh".to_string(),
            technique_family: "Exfiltration (T1048 - data transfer)".to_string(),
            macos_variant_id: "pb_macos_exfil_staging_ssh".to_string(),
            macos_variant_name: "Exfiltration: Staging + SSH Transfer".to_string(),
            slot_mapping: vec![
                "file_staging_activity".to_string(),
                "ssh_transfer_exec".to_string(),
            ],
            required_facts: vec![
                "FileWrite (/tmp, /var/tmp, or rare dir)".to_string(),
                "ProcExec (ssh with file args)".to_string(),
            ],
            status: "Candidate".to_string(),
            notes: "macOS: Limited visibility into network transfer; using file + SSH exec as proxy".to_string(),
            macos_observable: false,  // Network transfer not visible
            required_sources: vec!["openbsm".to_string()],
            classification: "candidate".to_string(),
        },
        PlaybookConversionEntry {
            linux_id: "pb_memfd_unsigned_egress".to_string(),
            technique_family: "Execution (T1203 - unsigned execution)".to_string(),
            macos_variant_id: "pb_macos_unsigned_exec_egress".to_string(),
            macos_variant_name: "Unsigned Execution + Immediate Egress".to_string(),
            slot_mapping: vec![
                "unsigned_or_quarantined_binary".to_string(),
                "first_seen_execution".to_string(),
                "network_activity_post_exec".to_string(),
            ],
            required_facts: vec![
                "ProcExec (unsigned codesign, or quarantine xattr)".to_string(),
                "NetConnect or lsof enrichment (post-exec)".to_string(),
            ],
            status: "Ground-Truth".to_string(),
            notes: "macOS: Codesign validation + first-seen + follow-on network (if enriched)".to_string(),
            macos_observable: true,
            required_sources: vec!["openbsm".to_string(), "codesign".to_string(), "lsof_snapshot".to_string()],
            classification: "ground_truth".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_matrix_populated() {
        let matrix = get_coverage_matrix();
        assert!(!matrix.is_empty());
        assert!(matrix.iter().all(|e| !e.linux_id.is_empty()));
        assert!(matrix.iter().all(|e| !e.macos_variant_id.is_empty()));
    }
}
