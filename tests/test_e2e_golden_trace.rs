// E2E Golden Trace Documentation
// Purpose: Document the 9 canonical event types and their triggering conditions

#[cfg(test)]
mod e2e_golden_trace {
    /// Golden trace example structure
    #[derive(Debug)]
    struct GoldenTrace {
        canonical_type: &'static str,
        os: &'static str,
        trigger: &'static str,
        base_category: &'static str,
    }

    // Define all 27 golden traces (9 types × 3 OS)
    const GOLDEN_TRACES: &[GoldenTrace] = &[
        // ============ CREDENTIAL_ACCESS ============
        GoldenTrace {
            canonical_type: "credential_access",
            os: "macOS",
            trigger: "cat ~/.ssh/id_rsa",
            base_category: "file_read",
        },
        GoldenTrace {
            canonical_type: "credential_access",
            os: "Linux",
            trigger: "cat /etc/shadow",
            base_category: "file_read",
        },
        GoldenTrace {
            canonical_type: "credential_access",
            os: "Windows",
            trigger: "Access LSASS process",
            base_category: "proc_access",
        },
        // ============ DISCOVERY ============
        GoldenTrace {
            canonical_type: "discovery",
            os: "macOS",
            trigger: "ps aux",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "discovery",
            os: "Linux",
            trigger: "ps aux",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "discovery",
            os: "Windows",
            trigger: "tasklist.exe",
            base_category: "process_exec",
        },
        // ============ EXFILTRATION ============
        GoldenTrace {
            canonical_type: "exfiltration",
            os: "macOS",
            trigger: "tar czf archive.tar.gz /data",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "exfiltration",
            os: "Linux",
            trigger: "tar czf archive.tar.gz /data",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "exfiltration",
            os: "Windows",
            trigger: "rar.exe a archive.rar c:\\data",
            base_category: "process_exec",
        },
        // ============ NETWORK_CONNECTION ============
        GoldenTrace {
            canonical_type: "network_connection",
            os: "macOS",
            trigger: "curl http://example.com",
            base_category: "network_connect",
        },
        GoldenTrace {
            canonical_type: "network_connection",
            os: "Linux",
            trigger: "curl http://example.com",
            base_category: "network_connect",
        },
        GoldenTrace {
            canonical_type: "network_connection",
            os: "Windows",
            trigger: "powershell.exe Invoke-WebRequest http://example.com",
            base_category: "network_connect",
        },
        // ============ PERSISTENCE_CHANGE ============
        GoldenTrace {
            canonical_type: "persistence_change",
            os: "macOS",
            trigger: "cp malicious.plist ~/Library/LaunchAgents/",
            base_category: "file_write",
        },
        GoldenTrace {
            canonical_type: "persistence_change",
            os: "Linux",
            trigger: "echo '0 * * * * /tmp/backdoor.sh' | crontab -",
            base_category: "file_write",
        },
        GoldenTrace {
            canonical_type: "persistence_change",
            os: "Windows",
            trigger: "reg add HKLM\\Software\\Run /v Malware /d c:\\malware.exe",
            base_category: "registry_write",
        },
        // ============ DEFENSE_EVASION ============
        GoldenTrace {
            canonical_type: "defense_evasion",
            os: "macOS",
            trigger: "defaults write com.apple.LaunchServices LSQuarantine -bool false",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "defense_evasion",
            os: "Linux",
            trigger: "rm /var/log/auth.log; rm /var/log/syslog",
            base_category: "file_delete",
        },
        GoldenTrace {
            canonical_type: "defense_evasion",
            os: "Windows",
            trigger: "wevtutil.exe cl Security",
            base_category: "process_exec",
        },
        // ============ PROCESS_INJECTION ============
        GoldenTrace {
            canonical_type: "process_injection",
            os: "macOS",
            trigger: "injector.bin --target=pid 1234",
            base_category: "proc_access",
        },
        GoldenTrace {
            canonical_type: "process_injection",
            os: "Linux",
            trigger: "ptrace(PTRACE_ATTACH, target_pid, ...)",
            base_category: "proc_access",
        },
        GoldenTrace {
            canonical_type: "process_injection",
            os: "Windows",
            trigger: "CreateRemoteThread(..., TARGET_PID, ...)",
            base_category: "proc_access",
        },
        // ============ AUTH_EVENT ============
        GoldenTrace {
            canonical_type: "auth_event",
            os: "macOS",
            trigger: "sudo -l",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "auth_event",
            os: "Linux",
            trigger: "sudo cat /etc/shadow",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "auth_event",
            os: "Windows",
            trigger: "runas.exe /user:SYSTEM cmd.exe",
            base_category: "process_exec",
        },
        // ============ SCRIPT_EXEC ============
        GoldenTrace {
            canonical_type: "script_exec",
            os: "macOS",
            trigger: "python3 payload.py",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "script_exec",
            os: "Linux",
            trigger: "bash /tmp/script.sh",
            base_category: "process_exec",
        },
        GoldenTrace {
            canonical_type: "script_exec",
            os: "Windows",
            trigger: "powershell.exe -ExecutionPolicy Bypass -File payload.ps1",
            base_category: "process_exec",
        },
    ];

    #[test]
    fn test_9_canonical_types_documented() {
        // Verify all 9 types are present
        let types: std::collections::HashSet<_> =
            GOLDEN_TRACES.iter().map(|t| t.canonical_type).collect();

        assert_eq!(types.len(), 9);
    }

    #[test]
    fn test_all_3_os_covered_per_type() {
        // Verify each type has traces for all 3 OS
        let mut type_os_count: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
            std::collections::HashMap::new();

        for trace in GOLDEN_TRACES {
            type_os_count
                .entry(trace.canonical_type)
                .or_insert_with(std::collections::HashSet::new)
                .insert(trace.os);
        }

        for (_type_name, os_set) in type_os_count {
            assert_eq!(os_set.len(), 3);
        }
    }

    #[test]
    fn test_trace_count_is_27() {
        // 9 types × 3 OS = 27 golden traces
        assert_eq!(GOLDEN_TRACES.len(), 27);
    }

    #[test]
    fn test_each_trace_has_valid_category() {
        // Verify all traces use known base categories
        let valid_categories = [
            "process_exec",
            "file_read",
            "file_write",
            "file_delete",
            "proc_access",
            "network_connect",
            "registry_write",
        ];

        for trace in GOLDEN_TRACES {
            assert!(valid_categories.contains(&trace.base_category));
        }
    }

    #[test]
    fn test_all_os_are_valid() {
        // Verify only 3 OS
        for trace in GOLDEN_TRACES {
            assert!(["macOS", "Linux", "Windows"].contains(&trace.os));
        }
    }
}
