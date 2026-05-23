use openaction::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct CounterSettings {
	step: i32,
	mode: String, // counter, timer, stopwatch
	initial: i32,
}

#[derive(Default)]
struct CounterState {
	value: i32,
	running: bool,
	start_time: Option<std::time::Instant>,
}

static STATES: LazyLock<Mutex<HashMap<InstanceId, CounterState>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct CounterAction;
#[async_trait]
impl Action for CounterAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.counter";
	type Settings = CounterSettings;

	async fn will_appear(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let mut states = STATES.lock().await;
		let state = states.entry(instance.instance_id.clone()).or_default();
		state.value = settings.initial;
		let _ = instance.set_title(Some(state.value.to_string()), None).await;
		Ok(())
	}

	async fn key_down(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let mut states = STATES.lock().await;
		let state = states.entry(instance.instance_id.clone()).or_default();

		match settings.mode.as_str() {
			"counter" => {
				state.value += settings.step;
				let _ = instance.set_title(Some(state.value.to_string()), None).await;
			}
			"timer" => {
				if !state.running {
					state.running = true;
					let instance_id = instance.instance_id.clone();
					let initial = state.value;
					tokio::spawn(async move {
						let mut remaining = initial;
						while remaining > 0 {
							tokio::time::sleep(std::time::Duration::from_secs(1)).await;
							remaining -= 1;
							if let Some(instance) = get_instance(instance_id.clone()).await {
								let _ = instance.set_title(Some(remaining.to_string()), None).await;
							}
							let mut states = STATES.lock().await;
							if let Some(s) = states.get_mut(&instance_id) {
								s.value = remaining;
								if !s.running {
									break;
								}
							}
						}
						if remaining == 0 {
							if let Some(instance) = get_instance(instance_id).await {
								let _ = instance.set_title(Some("DONE".to_string()), None).await;
							}
						}
					});
				} else {
					state.running = false;
				}
			}
			"stopwatch" => {
				if !state.running {
					state.running = true;
					state.start_time = Some(std::time::Instant::now());
					let instance_id = instance.instance_id.clone();
					tokio::spawn(async move {
						let start = std::time::Instant::now();
						loop {
							tokio::time::sleep(std::time::Duration::from_secs(1)).await;
							let elapsed = start.elapsed().as_secs() as i32;
							let mins = elapsed / 60;
							let secs = elapsed % 60;
							if let Some(instance) = get_instance(instance_id.clone()).await {
								let _ = instance.set_title(Some(format!("{:02}:{:02}", mins, secs)), None).await;
							}
							let mut states = STATES.lock().await;
							if let Some(s) = states.get_mut(&instance_id) {
								if !s.running {
									break;
								}
							}
						}
					});
				} else {
					state.running = false;
				}
			}
			_ => {}
		}
		Ok(())
	}

	async fn dial_rotate(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
		ticks: i16,
		_pressed: bool,
	) -> OpenActionResult<()> {
		if settings.mode == "counter" {
			let mut states = STATES.lock().await;
			let state = states.entry(instance.instance_id.clone()).or_default();
			state.value += ticks as i32 * settings.step;
			let _ = instance.set_title(Some(state.value.to_string()), None).await;
		}
		Ok(())
	}

	async fn dial_down(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		self.key_down(instance, settings).await
	}

	async fn dial_up(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		Ok(())
	}

	async fn will_disappear(
		&self,
		instance: &Instance,
		_settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let mut states = STATES.lock().await;
		states.remove(&instance.instance_id);
		Ok(())
	}
}
