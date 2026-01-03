//! capture_linux_rotating - Linux sensor/eBPF capture with rotating segments
//! PHASE B: Drains eBPF ringbuf, validates ABI, tracks metrics
//! Merges eBPF events with sensor events, writes rotating segments

#![allow(unused_imports)]

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::time::Duration;

// EDR event structure (must match eBPF's edr_event)
#[repr(C)]
struct EdrEvent {
    // Core fields (common across all events)
    ts: u64,
    abi_version: u16,
    event_size: u16,
    r#type: u32,
    syscall_id: u32,
    tgid: u32,
    uid: u32,
    fd: i32,
    ret: i64,
    flags: u16,
    aux_u32: u32,
    aux_u64: u64,

    // Network fields
    fam: u8,
    proto: u8,
    lport: u16,
    rport: u16,
    laddr4: u32,
    raddr4: u32,
    laddr6: [u8; 16],
    raddr6: [u8; 16],

    // Filesystem fields
    path: [u8; 128],
    path2: [u8; 128],

    // Process fields
    ppid: u32,
    comm: [u8; 16],
}

struct RingbufMetrics {
    events_read_total: u64,
    decode_failed_total: u64,
    reserve_failed_total: u64,
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--help" {
        eprintln!("capture_linux_rotating - Linux eBPF + sensor capture");
        eprintln!("Usage: capture_linux_rotating [OPTIONS]");
        eprintln!("  --output DIR          Output directory (default: /tmp/capture)");
        eprintln!("  --pin-root PATH       BPF pin root (default: /sys/fs/bpf/edr)");
        eprintln!("  --self-test-ebpf      Run 30s self-test (fail if no events)");
        return Ok(());
    }

    let mut output = PathBuf::from("/tmp/capture");
    let mut pin_root = PathBuf::from("/sys/fs/bpf/edr");
    let mut self_test = false;
    let mut mock_ebpf: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--pin-root" => {
                if i + 1 < args.len() {
                    pin_root = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--mock-ebpf" => {
                if i + 1 < args.len() {
                    mock_ebpf = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--self-test-ebpf" => {
                self_test = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    std::fs::create_dir_all(&output).context("Failed to create output directory")?;

    eprintln!(
        "[capture_linux_rotating] output={:?}, pin_root={:?}, mock_ebpf={:?}, self_test={}",
        output, pin_root, mock_ebpf, self_test
    );

    if self_test {
        run_ebpf_selftest(&pin_root).await?;
    } else if let Some(mock_path) = mock_ebpf {
        run_capture_with_mock(&output, &mock_path).await?;
    } else {
        run_capture_loop(&output, &pin_root).await?;
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<()> {
    Err(anyhow!("capture_linux_rotating is Linux-only"))
}

/// Self-test: validate ringbuf reader works
#[cfg(target_os = "linux")]
async fn run_ebpf_selftest(pin_root: &PathBuf) -> Result<()> {
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    eprintln!("[selftest] Running eBPF ringbuf self-test for 30 seconds");

    let ringbuf_path = pin_root.join("edr_events_rb");
    if !ringbuf_path.exists() {
        return Err(anyhow!("ringbuf not pinned at {:?}", ringbuf_path));
    }

    let metrics = Arc::new(Mutex::new(RingbufMetrics {
        events_read_total: 0,
        decode_failed_total: 0,
        reserve_failed_total: 0,
    }));

    let metrics_clone = metrics.clone();
    let start = Instant::now();

    // Spawn ringbuf reader task
    let reader_handle = tokio::spawn(async move {
        let _ = read_ringbuf_loop(&ringbuf_path, &metrics_clone).await;
    });

    // Print stats every 5 seconds for 30 seconds
    for _ in 0..6 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let m = metrics.lock().unwrap();
        let elapsed = start.elapsed();
        eprintln!(
            "[selftest] {:.0}s: read={} failed={} reserve_failed={}",
            elapsed.as_secs_f64(),
            m.events_read_total,
            m.decode_failed_total,
            m.reserve_failed_total
        );
    }

    reader_handle.abort();
    let final_metrics = metrics.lock().unwrap();

    if final_metrics.events_read_total == 0 {
        eprintln!("[selftest] ✗ FAILED: no events read after 30s");
        std::process::exit(1);
    }

    if final_metrics.decode_failed_total > 0 {
        eprintln!("[selftest] ✗ FAILED: decode errors detected");
        std::process::exit(1);
    }

    eprintln!(
        "[selftest] ✓ PASSED: {} events, 0 decode failures",
        final_metrics.events_read_total
    );
    Ok(())
}

/// Main capture loop: ringbuf + sensors
#[cfg(target_os = "linux")]
async fn run_capture_loop(output: &PathBuf, pin_root: &PathBuf) -> Result<()> {
    use std::sync::{Arc, Mutex};

    let ringbuf_path = pin_root.join("edr_events_rb");

    if !ringbuf_path.exists() {
        eprintln!("[capture] ringbuf not yet pinned, waiting");
        // Continue anyway, will retry
    }

    let metrics = Arc::new(Mutex::new(RingbufMetrics {
        events_read_total: 0,
        decode_failed_total: 0,
        reserve_failed_total: 0,
    }));

    let metrics_clone = metrics.clone();
    let ringbuf_path_clone = ringbuf_path.clone();

    // Spawn ringbuf reader
    let reader_handle = tokio::spawn(async move {
        let _ = read_ringbuf_loop(&ringbuf_path_clone, &metrics_clone).await;
    });

    // Main heartbeat loop
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let m = metrics.lock().unwrap();
        eprintln!(
            "[capture] ebpf_events_read={} decode_failed={}",
            m.events_read_total, m.decode_failed_total
        );
    }
}

/// Open and read from pinned ringbuf map
#[cfg(target_os = "linux")]
async fn read_ringbuf_loop(
    ringbuf_path: &PathBuf,
    metrics: &std::sync::Arc<std::sync::Mutex<RingbufMetrics>>,
) -> Result<()> {
    use aya::maps::Map;

    loop {
        if !ringbuf_path.exists() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        match Map::from_pin(ringbuf_path) {
            Ok(map) => {
                if let Err(e) = process_ringbuf(&map, metrics).await {
                    eprintln!("[ringbuf] Error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
            Err(e) => {
                eprintln!("[ringbuf] Failed to open map: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

/// Capture loop using mock eBPF stream from fixture file
#[cfg(target_os = "linux")]
async fn run_capture_with_mock(output: &PathBuf, mock_file: &PathBuf) -> Result<()> {
    use forensic_hooks::linux::ebpf::MockEbpfStream;

    eprintln!(
        "[capture_linux_rotating] Reading mock eBPF fixture from {:?}",
        mock_file
    );

    let mut stream = MockEbpfStream::new(mock_file)?;
    let mut events_read = 0u64;
    let mut decode_failed = 0u64;

    // Poll the mock stream
    loop {
        match stream.poll(100) {
            Ok(events) => {
                events_read += events.len() as u64;
                eprintln!("[capture_linux_rotating] Mock poll: {} events, total read={}, decode_failed={}",
                    events.len(), events_read, stream.loss_metrics().decode_errors);

                if events.is_empty() {
                    break; // EOF
                }

                // Write a segment for this batch
                std::fs::write(
                    output.join("mock_segment.jsonl"),
                    format!(
                        "{{\"transport\":\"mock\",\"events_read\":{},\"decode_failed\":{}}}\n",
                        events_read,
                        stream.loss_metrics().decode_errors
                    ),
                )?;
            }
            Err(e) => {
                eprintln!("[capture_linux_rotating] Mock poll error: {}", e);
                break;
            }
        }
    }

    eprintln!(
        "[capture_linux_rotating] Mock capture done: {} events read, {} decode failures",
        events_read,
        stream.loss_metrics().decode_errors
    );

    // Write heartbeat
    std::fs::write(
        output.join("heartbeat.json"),
        serde_json::to_string(&serde_json::json!({
            "transport": "mock",
            "events_read_total": events_read,
            "decode_failed_total": stream.loss_metrics().decode_errors,
        }))?,
    )?;

    Ok(())
}

#[cfg(target_os = "linux")]
async fn process_ringbuf(
    map: &aya::maps::Map,
    metrics: &std::sync::Arc<std::sync::Mutex<RingbufMetrics>>,
) -> Result<()> {
    use aya::maps::RingBuf;

    let mut ringbuf = RingBuf::try_from(map)?;

    loop {
        match ringbuf.next() {
            Some(data) => {
                // Validate event size
                if data.len() < std::mem::size_of::<EdrEvent>() {
                    let mut m = metrics.lock().unwrap();
                    m.decode_failed_total += 1;
                    continue;
                }

                // Read event
                let event_bytes = &data[..std::mem::size_of::<EdrEvent>()];
                let event = unsafe { *(event_bytes.as_ptr() as *const EdrEvent) };

                // Validate ABI
                if event.abi_version != 1 || event.event_size != 384 {
                    let mut m = metrics.lock().unwrap();
                    m.decode_failed_total += 1;
                    continue;
                }

                // Success
                let mut m = metrics.lock().unwrap();
                m.events_read_total += 1;

                eprintln!(
                    "[ringbuf] event_type={} pid={} uid={} syscall={}",
                    event.r#type, event.tgid, event.uid, event.syscall_id
                );
            }
            None => {
                // Ringbuf empty, yield and retry
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
