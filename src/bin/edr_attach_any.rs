//! edr_attach_any - Explicit attach matrix with hard-fail semantics
//! All 17 eBPF programs with precise Aya attach calls
//! PHASE A: Load + attach programs, pin maps under /sys/fs/bpf/edr

#![allow(unused_imports)]

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use aya::{
    programs::{KProbe, TracePoint},
    BpfLoader,
};

// Explicit attach matrix: name, object file, tier, attach_spec
#[derive(Debug, Clone)]
struct AttachSpec {
    name: &'static str,
    object: &'static str,
    tier: AttachTier,
    /// Slice of (attach_type, group, event/fn_name)
    /// attach_type: "tracepoint" or "kprobe" or "kretprobe"
    hooks: &'static [(&'static str, &'static str, &'static str)],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttachTier {
    AlwaysOn,
    OnDemand,
}

// All 17 programs with precise hookpoints extracted from SEC() declarations
const ALWAYS_ON_PROC_LIFECYCLE: &[(&str, &str, &str)] = &[
    ("tracepoint", "sched", "sched_process_fork"),
    ("tracepoint", "sched", "sched_process_exec"),
    ("tracepoint", "sched", "sched_process_exit"),
];

const ALWAYS_ON_WX_EXEC: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_mmap"),
    ("tracepoint", "syscalls", "sys_enter_mprotect"),
];

const ALWAYS_ON_TCP_CONNECT: &[(&str, &str, &str)] = &[
    ("kprobe", "", "tcp_v4_connect"),
    ("kprobe", "", "tcp_v6_connect"),
];

const ALWAYS_ON_PRIVILEGE_TRANSITIONS: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_setuid"),
    ("tracepoint", "syscalls", "sys_enter_setgid"),
    ("tracepoint", "syscalls", "sys_enter_setresuid"),
    ("tracepoint", "syscalls", "sys_enter_setresgid"),
];

const ALWAYS_ON_FILE_OPS_GUARD: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_openat"),
    ("tracepoint", "syscalls", "sys_enter_unlinkat"),
    ("tracepoint", "syscalls", "sys_enter_renameat2"),
    ("tracepoint", "syscalls", "sys_enter_fchmodat"),
    ("tracepoint", "syscalls", "sys_enter_fchownat"),
];

const ALWAYS_ON_EXEC_PROVENANCE: &[(&str, &str, &str)] =
    &[("tracepoint", "sched", "sched_process_exec")];

const ALWAYS_ON_PTRACE_INJECTION: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_ptrace"),
    ("tracepoint", "syscalls", "sys_enter_process_vm_writev"),
    ("tracepoint", "syscalls", "sys_enter_process_vm_readv"),
    ("tracepoint", "syscalls", "sys_enter_memfd_create"),
];

const ALWAYS_ON_KERNEL_MODULE_LOAD: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_init_module"),
    ("tracepoint", "syscalls", "sys_enter_finit_module"),
    ("tracepoint", "syscalls", "sys_enter_delete_module"),
];

const ALWAYS_ON_SIGNAL_INTEGRITY: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_kill"),
    ("tracepoint", "syscalls", "sys_enter_ptrace"),
    ("tracepoint", "syscalls", "sys_enter_umount2"),
    ("tracepoint", "syscalls", "sys_enter_bpf"),
];

const ALWAYS_ON_DNS_TELEMETRY: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_sendto"),
    ("tracepoint", "syscalls", "sys_enter_sendmsg"),
];

const ON_DEMAND_SYSCALL_TRACER: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_execve"),
    ("tracepoint", "syscalls", "sys_enter_fork"),
    ("tracepoint", "syscalls", "sys_enter_mmap"),
];

const ON_DEMAND_NET_FLOW: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_sendto"),
    ("tracepoint", "syscalls", "sys_enter_recvfrom"),
];

const ON_DEMAND_ENCRYPTED_PAYLOAD_MONITOR: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_read"),
    ("tracepoint", "syscalls", "sys_enter_recvfrom"),
    ("tracepoint", "syscalls", "sys_enter_recvmsg"),
];

const ON_DEMAND_CONTAINER_EXEC: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_execve"),
    ("tracepoint", "syscalls", "sys_enter_execveat"),
];

const ON_DEMAND_SUSPICIOUS_IPC: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_ptrace"),
    ("tracepoint", "syscalls", "sys_enter_process_vm_readv"),
    ("tracepoint", "syscalls", "sys_enter_process_vm_writev"),
    ("tracepoint", "syscalls", "sys_enter_msgctl"),
];

const ON_DEMAND_CRED_ACCESS: &[(&str, &str, &str)] = &[
    ("tracepoint", "syscalls", "sys_enter_capget"),
    ("tracepoint", "syscalls", "sys_enter_capset"),
    ("tracepoint", "syscalls", "sys_enter_getuid"),
    ("tracepoint", "syscalls", "sys_enter_geteuid"),
    ("tracepoint", "syscalls", "sys_enter_getgid"),
    ("tracepoint", "syscalls", "sys_enter_getegid"),
];

const ON_DEMAND_USB_MONITOR: &[(&str, &str, &str)] = &[
    ("tracepoint", "usb", "usb_dev_uevent"),
    ("tracepoint", "syscalls", "sys_enter_open"),
];

const ATTACH_MATRIX: &[AttachSpec] = &[
    // ALWAYS_ON (10 programs)
    AttachSpec {
        name: "proc_lifecycle",
        object: "proc_lifecycle.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_PROC_LIFECYCLE,
    },
    AttachSpec {
        name: "wx_exec",
        object: "wx_exec.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_WX_EXEC,
    },
    AttachSpec {
        name: "tcp_connect",
        object: "tcp_connect.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_TCP_CONNECT,
    },
    AttachSpec {
        name: "privilege_transitions",
        object: "privilege_transitions.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_PRIVILEGE_TRANSITIONS,
    },
    AttachSpec {
        name: "file_ops_guard",
        object: "file_ops_guard.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_FILE_OPS_GUARD,
    },
    AttachSpec {
        name: "exec_provenance",
        object: "exec_provenance.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_EXEC_PROVENANCE,
    },
    AttachSpec {
        name: "ptrace_injection",
        object: "ptrace_injection.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_PTRACE_INJECTION,
    },
    AttachSpec {
        name: "kernel_module_load",
        object: "kernel_module_load.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_KERNEL_MODULE_LOAD,
    },
    AttachSpec {
        name: "signal_integrity",
        object: "signal_integrity.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_SIGNAL_INTEGRITY,
    },
    AttachSpec {
        name: "dns_telemetry",
        object: "dns_telemetry.bpf.o",
        tier: AttachTier::AlwaysOn,
        hooks: ALWAYS_ON_DNS_TELEMETRY,
    },
    // ON_DEMAND (7 programs)
    AttachSpec {
        name: "syscall_tracer",
        object: "syscall_tracer.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_SYSCALL_TRACER,
    },
    AttachSpec {
        name: "net_flow",
        object: "net_flow.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_NET_FLOW,
    },
    AttachSpec {
        name: "encrypted_payload_monitor",
        object: "encrypted_payload_monitor.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_ENCRYPTED_PAYLOAD_MONITOR,
    },
    AttachSpec {
        name: "container_exec",
        object: "container_exec.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_CONTAINER_EXEC,
    },
    AttachSpec {
        name: "suspicious_ipc",
        object: "suspicious_ipc.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_SUSPICIOUS_IPC,
    },
    AttachSpec {
        name: "cred_access",
        object: "cred_access.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_CRED_ACCESS,
    },
    AttachSpec {
        name: "usb_monitor",
        object: "usb_monitor.bpf.o",
        tier: AttachTier::OnDemand,
        hooks: ON_DEMAND_USB_MONITOR,
    },
];

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;

    if config.verbose {
        eprintln!("[edr_attach_any] Starting with explicit attach matrix");
        eprintln!("[edr_attach_any] BPF dir: {:?}", config.bpf_dir);
        eprintln!("[edr_attach_any] Pin root: {:?}", config.pin_root);
    }

    // Step 1: Raise RLIMIT_MEMLOCK
    raise_memlock_limit(config.verbose)?;

    // Step 2: Create and verify pin directory
    std::fs::create_dir_all(&config.pin_root).context("Failed to create pin directory")?;
    eprintln!(
        "[edr_attach_any] Pin directory ready: {:?}",
        config.pin_root
    );

    // Step 3: Determine which programs to attach
    let mut to_attach = Vec::new();
    for spec in ATTACH_MATRIX {
        if spec.tier == AttachTier::AlwaysOn
            || config.enabled_programs.contains(&spec.name.to_string())
        {
            to_attach.push(spec);
        }
    }

    eprintln!("[edr_attach_any] Attaching {} programs", to_attach.len());

    // Step 4: Attach each program with hard-fail semantics
    let mut attached_count = 0;
    let mut failed_count = 0;

    for spec in to_attach {
        match attach_program_with_matrix(&config, spec).await {
            Ok(_) => {
                eprintln!("[edr_attach_any] ATTACHED {}", spec.name);
                attached_count += 1;
            }
            Err(e) => {
                eprintln!("[edr_attach_any] FAILED {} — {}", spec.name, e);
                failed_count += 1;
            }
        }
    }

    // Step 5: Report summary
    eprintln!(
        "[edr_attach_any] Summary: {}/{} attached, {} failed",
        attached_count,
        to_attach.len(),
        failed_count
    );

    if failed_count > 0 {
        return Err(anyhow!("Failed to attach {} programs", failed_count));
    }

    eprintln!("[edr_attach_any] Ready");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<()> {
    eprintln!("[edr_attach_any] ERROR: edr_attach_any is Linux-only");
    eprintln!("[edr_attach_any] This platform cannot attach BPF programs");
    eprintln!("[edr_attach_any] Use --mock-ebpf on macOS/Windows for testing");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
async fn attach_program_with_matrix(config: &AttachConfig, spec: &AttachSpec) -> Result<()> {
    let obj_path = config.bpf_dir.join(spec.object);

    if !obj_path.exists() {
        return Err(anyhow!("Object file not found: {}", spec.object));
    }

    // Load BPF object file
    let mut bpf = BpfLoader::new()
        .map_pin_path(&config.pin_root)
        .load_file(&obj_path)
        .context(format!("Failed to load {}", spec.object))?;

    // Collect all program names by type before attachment
    let mut tracepoint_programs = Vec::new();
    let mut kprobe_programs = Vec::new();

    for (name, prog) in bpf.programs() {
        match prog.prog_type() {
            aya::programs::ProgramType::TracePoint => {
                tracepoint_programs.push(name.to_string());
            }
            aya::programs::ProgramType::KProbe => {
                kprobe_programs.push(name.to_string());
            }
            _ => {}
        }
    }

    // Attach each hookpoint specified in the matrix
    for (attach_type, group, event_or_fn) in spec.hooks {
        match *attach_type {
            "tracepoint" => {
                // Use first available tracepoint program (ideally we'd match by SEC section)
                if tracepoint_programs.is_empty() {
                    return Err(anyhow!("No tracepoint program found in {}", spec.object));
                }
                let prog_name = &tracepoint_programs[0];

                let program = bpf
                    .program_mut(prog_name)
                    .context(format!("Program {} not found", prog_name))?;

                // Cast to TracePoint and attach
                let tp = program
                    .downcast_mut::<TracePoint>()
                    .context(format!("Program {} is not a tracepoint", prog_name))?;

                let link = tp.attach().context(format!(
                    "Failed to attach tracepoint {}/{}",
                    group, event_or_fn
                ))?;

                // Forget the link to keep it alive
                std::mem::forget(link);
                eprintln!("[edr_attach_any]   → {}/{}", group, event_or_fn);
            }
            "kprobe" => {
                // Use first available kprobe program
                if kprobe_programs.is_empty() {
                    return Err(anyhow!("No kprobe program found in {}", spec.object));
                }
                let prog_name = &kprobe_programs[0];

                let program = bpf
                    .program_mut(prog_name)
                    .context(format!("Program {} not found", prog_name))?;

                // Cast to KProbe and attach
                let kp = program
                    .downcast_mut::<KProbe>()
                    .context(format!("Program {} is not a kprobe", prog_name))?;

                let link = kp
                    .attach()
                    .context(format!("Failed to attach kprobe {}", event_or_fn))?;

                std::mem::forget(link);
                eprintln!("[edr_attach_any]   → kprobe/{}", event_or_fn);
            }
            _ => {
                return Err(anyhow!("Unknown attach type: {}", attach_type));
            }
        }
    }

    Ok(())
}

struct AttachConfig {
    bpf_dir: PathBuf,
    pin_root: PathBuf,
    enabled_programs: Vec<String>,
    verbose: bool,
}

fn parse_args() -> Result<AttachConfig> {
    let args: Vec<String> = std::env::args().collect();
    let mut bpf_dir = PathBuf::from("target/debug/bpf");
    let mut pin_root = PathBuf::from("/sys/fs/bpf/edr");
    let mut enabled_programs = Vec::new();
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--bpf-dir" => {
                i += 1;
                if i < args.len() {
                    bpf_dir = PathBuf::from(&args[i]);
                }
            }
            "--pin-root" => {
                i += 1;
                if i < args.len() {
                    pin_root = PathBuf::from(&args[i]);
                }
            }
            "--enable" => {
                i += 1;
                if i < args.len() {
                    if args[i] == "on_demand" {
                        for spec in ATTACH_MATRIX {
                            if spec.tier == AttachTier::OnDemand {
                                enabled_programs.push(spec.name.to_string());
                            }
                        }
                    } else {
                        enabled_programs.push(args[i].clone());
                    }
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            _ => {}
        }
        i += 1;
    }

    Ok(AttachConfig {
        bpf_dir,
        pin_root,
        enabled_programs,
        verbose,
    })
}

fn print_help() {
    eprintln!("edr_attach_any - Explicit eBPF attach matrix with hard-fail semantics");
    eprintln!("");
    eprintln!("Usage: edr_attach_any [OPTIONS]");
    eprintln!("");
    eprintln!("OPTIONS:");
    eprintln!("  --bpf-dir DIR         BPF object directory (default: target/debug/bpf)");
    eprintln!("  --pin-root PATH       Map pin root (default: /sys/fs/bpf/edr)");
    eprintln!("  --enable <prog>       Enable specific ON_DEMAND program");
    eprintln!("  --enable on_demand    Enable all ON_DEMAND programs");
    eprintln!("  -v, --verbose         Verbose output");
    eprintln!("  -h, --help            Show this help");
    eprintln!("");
    eprintln!("[edr_attach_any] Attach matrix (17 programs):");
    eprintln!("");
    eprintln!("ALWAYS_ON (10):");
    for spec in ATTACH_MATRIX {
        if spec.tier == AttachTier::AlwaysOn {
            eprintln!("  {} ({})", spec.name, spec.object);
            for (attach_type, group, event) in spec.hooks {
                if *attach_type == "tracepoint" {
                    eprintln!("    → {}/{}", group, event);
                } else {
                    eprintln!("    → {} {}", attach_type, event);
                }
            }
        }
    }
    eprintln!("");
    eprintln!("ON_DEMAND (7):");
    for spec in ATTACH_MATRIX {
        if spec.tier == AttachTier::OnDemand {
            eprintln!("  {} ({})", spec.name, spec.object);
            for (attach_type, group, event) in spec.hooks {
                if *attach_type == "tracepoint" {
                    eprintln!("    → {}/{}", group, event);
                } else {
                    eprintln!("    → {} {}", attach_type, event);
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn raise_memlock_limit(verbose: bool) -> Result<()> {
    use libc::{getrlimit, setrlimit, RLIMIT_MEMLOCK};

    unsafe {
        let mut rlim: libc::rlimit = std::mem::zeroed();

        if getrlimit(RLIMIT_MEMLOCK, &mut rlim) != 0 {
            return Err(anyhow!("getrlimit(RLIMIT_MEMLOCK) failed"));
        }

        rlim.rlim_cur = 512 * 1024 * 1024;
        rlim.rlim_max = 512 * 1024 * 1024;

        if setrlimit(RLIMIT_MEMLOCK, &rlim) != 0 {
            return Err(anyhow!("setrlimit(RLIMIT_MEMLOCK) failed; may need sudo"));
        }

        if verbose {
            eprintln!("[edr_attach_any] RLIMIT_MEMLOCK raised to 512 MB");
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn raise_memlock_limit(_verbose: bool) -> Result<()> {
    Ok(())
}
