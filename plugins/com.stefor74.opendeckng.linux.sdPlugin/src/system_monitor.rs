use openaction::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SystemMonitorSettings {
	metric: String, // cpu, ram, net_up, net_down
	interval: u64,  // seconds, default 2
}

static TASK_RUNNING: AtomicBool = AtomicBool::new(false);
static ACTIVE_INSTANCES: LazyLock<Mutex<Vec<(InstanceId, String, u64)>>> = LazyLock::new(|| Mutex::new(Vec::new()));

async fn read_cpu_usage() -> Result<f32, anyhow::Error> {
	let stat = tokio::fs::read_to_string("/proc/stat").await?;
	let first_line = stat.lines().next().unwrap_or("");
	let parts: Vec<u64> = first_line
		.split_whitespace()
		.skip(1)
		.filter_map(|s| s.parse().ok())
		.collect();
	if parts.len() >= 4 {
		let idle = parts[3];
		let total: u64 = parts.iter().sum();
		let non_idle = total - idle;
		if total > 0 {
			return Ok((non_idle as f32 / total as f32) * 100.0);
		}
	}
	Ok(0.0)
}

async fn read_ram_usage() -> Result<f32, anyhow::Error> {
	let meminfo = tokio::fs::read_to_string("/proc/meminfo").await?;
	let mut total = 0u64;
	let mut available = 0u64;
	for line in meminfo.lines() {
		if line.starts_with("MemTotal:") {
			total = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
		} else if line.starts_with("MemAvailable:") {
			available = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
		}
	}
	if total > 0 {
		let used = total - available;
		return Ok((used as f32 / total as f32) * 100.0);
	}
	Ok(0.0)
}

async fn read_net_speed(direction: &str) -> Result<f32, anyhow::Error> {
	let dev = tokio::fs::read_to_string("/proc/net/dev").await?;
	let mut total_bytes = 0u64;
	for line in dev.lines().skip(2) {
		let cols: Vec<&str> = line.split_whitespace().collect();
		if cols.len() >= 9 && !cols[0].starts_with("lo") {
			if direction == "up" {
				total_bytes += cols.get(9).and_then(|s| s.parse().ok()).unwrap_or(0u64);
			} else {
				total_bytes += cols.get(1).and_then(|s| s.parse().ok()).unwrap_or(0u64);
			}
		}
	}
	// Return in MB/s - this is a simplified reading, for accurate speed we'd need delta
	Ok((total_bytes / 1024 / 1024) as f32)
}

async fn update_loop() {
	loop {
		let instances = ACTIVE_INSTANCES.lock().await.clone();
		if instances.is_empty() {
			TASK_RUNNING.store(false, Ordering::SeqCst);
			break;
		}

		for (instance_id, metric, _interval) in &instances {
			let value = match metric.as_str() {
				"cpu" => read_cpu_usage().await.map(|v| format!("CPU: {:.0}%", v)),
				"ram" => read_ram_usage().await.map(|v| format!("RAM: {:.0}%", v)),
				"net_up" => read_net_speed("up").await.map(|v| format!("Up: {:.0}MB", v)),
				"net_down" => read_net_speed("down").await.map(|v| format!("Down: {:.0}MB", v)),
				_ => Ok(String::new()),
			};

			if let Ok(text) = value {
				if let Some(instance) = get_instance(instance_id.clone()).await {
					let _ = instance.set_title(Some(text), None).await;
				}
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
	}
}

async fn start_if_needed() {
	if !TASK_RUNNING.swap(true, Ordering::SeqCst) {
		tokio::spawn(update_loop());
	}
}

pub struct SystemMonitorAction;
#[async_trait]
impl Action for SystemMonitorAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.systemmonitor";
	type Settings = SystemMonitorSettings;

	async fn will_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let instance_id = instance.instance_id.clone();
		let metric = settings.metric.clone();
		let interval = settings.interval.max(1);

		ACTIVE_INSTANCES.lock().await.push((instance_id, metric, interval));
		start_if_needed().await;
		Ok(())
	}

	async fn will_disappear(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let mut instances = ACTIVE_INSTANCES.lock().await;
		instances.retain(|(id, _, _)| id != &instance.instance_id);
		Ok(())
	}
}
