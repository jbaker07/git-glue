// windows/sensors/network_monitor.rs
// Network monitoring (DNS, SMB, named pipes, suspicious connections)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub ts: DateTime<Utc>,
    pub event_type: NetworkEventType,
    pub src_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_ip: Option<String>,
    pub dst_port: Option<u16>,
    pub protocol: String,  // tcp, udp, icmp, dns
    pub domain: Option<String>,
    pub pid: Option<i32>,
    pub exe: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEventType {
    DnsQuery,
    TcpConnect,
    UdpSend,
    IcmpEcho,
    SmbShare,
    NamedPipe,
}

#[derive(Debug)]
pub struct NetworkMonitor {
    suspicious_ports: Vec<u16>,
    suspicious_domains: Vec<String>,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMonitor {
    pub fn new() -> Self {
        let suspicious_ports = vec![
            445,   // SMB
            139,   // NetBIOS
            135,   // RPC
            5985,  // WinRM
            5986,  // WinRM SSL
            3389,  // RDP
            389,   // LDAP
            636,   // LDAPS
            3306,  // MySQL
            5432,  // PostgreSQL
            27017, // MongoDB
        ];

        Self {
            suspicious_ports,
            suspicious_domains: vec![],
        }
    }

    pub fn is_suspicious_connection(&self, event: &NetworkEvent) -> bool {
        // SMB/RPC to unusual hosts (lateral movement)
        if let Some(port) = event.dst_port {
            if (port == 445 || port == 139 || port == 135) {
                if let Some(dst_ip) = &event.dst_ip {
                    // Private network connection is normal; public is suspicious
                    if !is_private_ip(dst_ip) && !is_localhost(dst_ip) {
                        return true;
                    }
                }
            }
        }

        // DNS to non-standard resolver
        if event.event_type == NetworkEventType::DnsQuery {
            if let Some(port) = event.dst_port {
                if port != 53 && port != 5353 {
                    return true;
                }
            }
        }

        // High port range connection from system process (unusual)
        if let Some(port) = event.dst_port {
            if port > 40000 && is_system_process(&event.exe) {
                return true;
            }
        }

        false
    }

    pub fn to_canonical_event(&self, event: &NetworkEvent) -> Option<crate::Event> {
        if !self.is_suspicious_connection(event) {
            return None;
        }

        let exe = event.exe.clone();
        let pid = event.pid;
        
        let sock = crate::event::SockMeta {
            family: Some(2), // AF_INET
            laddr: event.src_ip.clone(),
            lport: event.src_port,
            raddr: event.dst_ip.clone(),
            rport: event.dst_port,
        };

        Some(crate::Event {
            ts: event.ts,
            event_type: crate::event::EventType::Other,
            pid,
            exe,
            sock: Some(sock),
            tags: vec![
                "windows_network".to_string(),
                format!("{:?}", event.event_type).to_lowercase(),
                event.protocol.clone(),
                event.domain.clone().unwrap_or_default(),
            ],
            ..Default::default()
        })
    }
}

fn is_private_ip(ip: &str) -> bool {
    ip.starts_with("10.") || 
    ip.starts_with("172.16.") || 
    ip.starts_with("192.168.") ||
    ip.starts_with("169.254.")  // APIPA
}

fn is_localhost(ip: &str) -> bool {
    ip == "127.0.0.1" || ip == "::1" || ip == "localhost"
}

fn is_system_process(exe: &Option<String>) -> bool {
    match exe {
        Some(e) => {
            let lower = e.to_lowercase();
            lower.contains("svchost.exe") ||
            lower.contains("system.exe") ||
            lower.contains("lsass.exe") ||
            lower.contains("csrss.exe")
        },
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_ip() {
        assert!(is_private_ip("192.168.1.1"));
        assert!(is_private_ip("10.0.0.1"));
        assert!(!is_private_ip("8.8.8.8"));
    }

    #[test]
    fn test_suspicious_smb_external() {
        let monitor = NetworkMonitor::new();
        let event = NetworkEvent {
            ts: Utc::now(),
            event_type: NetworkEventType::SmbShare,
            src_ip: Some("192.168.1.100".to_string()),
            src_port: Some(49152),
            dst_ip: Some("8.8.8.8".to_string()),
            dst_port: Some(445),
            protocol: "tcp".to_string(),
            domain: None,
            pid: Some(1234),
            exe: Some("cmd.exe".to_string()),
            user: Some("admin".to_string()),
        };
        assert!(monitor.is_suspicious_connection(&event));
    }
}
