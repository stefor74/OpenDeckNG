use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct MediaControlSettings {
	action: String, // playpause, next, previous, mute
}

async fn run_media_action(action: &str) -> Result<(), anyhow::Error> {
	let (cmd, args) = match action {
		"playpause" => ("playerctl", vec!["play-pause"]),
		"next" => ("playerctl", vec!["next"]),
		"previous" => ("playerctl", vec!["previous"]),
		"stop" => ("playerctl", vec!["stop"]),
		"mute" => ("pactl", vec!["set-sink-mute", "@DEFAULT_SINK@", "toggle"]),
		_ => return Ok(()),
	};

	let output = std::process::Command::new(cmd)
		.args(&args)
		.output()?;

	if !output.status.success() && action != "mute" {
		// Fallback: try dbus-send for systems without playerctl
		let dbus_cmd = match action {
			"playpause" => "PlayPause",
			"next" => "Next",
			"previous" => "Previous",
			"stop" => "Stop",
			_ => return Ok(()),
		};
		let _ = std::process::Command::new("dbus-send")
			.args([
				"--type=method_call",
				"--dest=org.mpris.MediaPlayer2.playerctld",
				"/org/mpris/MediaPlayer2",
				&format!("org.mpris.MediaPlayer2.Player.{}", dbus_cmd),
			])
			.output()?;
	}

	Ok(())
}

pub struct MediaControlAction;
#[async_trait]
impl Action for MediaControlAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.mediacontrol";
	type Settings = MediaControlSettings;

	async fn key_down(
		&self,
		_instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let action = settings.action.clone();
		tokio::spawn(async move {
			if let Err(error) = run_media_action(&action).await {
				log::warn!("Failed to run media action: {error}");
			}
		});
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
}
