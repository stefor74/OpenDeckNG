use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AudioVolumeSettings {
	step: u8,
	show_volume: bool,
}

fn get_volume_command() -> &'static str {
	if std::process::Command::new("wpctl")
		.arg("--version")
		.output()
		.map(|o| o.status.success())
		.unwrap_or(false)
	{
		"wpctl"
	} else {
		"pactl"
	}
}

async fn get_current_volume() -> Result<u8, anyhow::Error> {
	let cmd = get_volume_command();
	if cmd == "wpctl" {
		let output = std::process::Command::new("wpctl")
			.args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
			.output()?;
		let text = String::from_utf8_lossy(&output.stdout);
		// Output: "Volume: 0.65"
		if let Some(vol) = text.split_whitespace().nth(1) {
			if let Ok(v) = vol.parse::<f32>() {
				return Ok((v * 100.0).clamp(0.0, 100.0) as u8);
			}
		}
	} else {
		let output = std::process::Command::new("pactl")
			.args(["list", "sinks"])
			.output()?;
		let text = String::from_utf8_lossy(&output.stdout);
		for line in text.lines() {
			if line.contains("Volume:") && line.contains("front-left") {
				if let Some(percent) = line.split('%').next() {
					if let Some(num) = percent.rsplit(' ').next() {
						if let Ok(v) = num.parse::<u8>() {
							return Ok(v);
						}
					}
				}
			}
		}
	}
	Ok(50)
}

async fn set_volume(delta: i16) -> Result<(), anyhow::Error> {
	let current = get_current_volume().await?;
	let new = (current as i16 + delta).clamp(0, 100) as u8;

	let cmd = get_volume_command();
	if cmd == "wpctl" {
		std::process::Command::new("wpctl")
			.args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", new as f32 / 100.0)])
			.output()?;
	} else {
		std::process::Command::new("pactl")
			.args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", new)])
			.output()?;
	}
	Ok(())
}

async fn mute_toggle() -> Result<(), anyhow::Error> {
	let cmd = get_volume_command();
	if cmd == "wpctl" {
		std::process::Command::new("wpctl")
			.args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
			.output()?;
	} else {
		std::process::Command::new("pactl")
			.args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
			.output()?;
	}
	Ok(())
}

pub struct AudioVolumeAction;
#[async_trait]
impl Action for AudioVolumeAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.audiovolume";
	type Settings = AudioVolumeSettings;

	async fn key_down(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let instance_id = instance.instance_id.clone();
		let show_volume = settings.show_volume;
		tokio::spawn(async move {
			if let Err(error) = mute_toggle().await {
				log::warn!("Failed to toggle mute: {error}");
				return;
			}
			if show_volume {
				if let Ok(vol) = get_current_volume().await {
					if let Some(instance) = get_instance(instance_id).await {
						let _ = instance.set_title(Some(format!("Vol: {}%", vol)), None).await;
					}
				}
			}
		});
		Ok(())
	}

	async fn dial_rotate(
		&self,
		instance: &Instance,
		settings: &Self::Settings,
		ticks: i16,
		_pressed: bool,
	) -> OpenActionResult<()> {
		let instance_id = instance.instance_id.clone();
		let step = settings.step.max(1) as i16;
		let show_volume = settings.show_volume;
		let delta = ticks * step;

		tokio::spawn(async move {
			if let Err(error) = set_volume(delta).await {
				log::warn!("Failed to set volume: {error}");
				return;
			}
			if show_volume {
				if let Ok(vol) = get_current_volume().await {
					if let Some(instance) = get_instance(instance_id).await {
						let _ = instance.set_title(Some(format!("Vol: {}%", vol)), None).await;
					}
				}
			}
		});
		Ok(())
	}
}
