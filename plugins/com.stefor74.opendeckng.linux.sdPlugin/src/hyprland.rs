use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct HyprlandWorkspaceSettings {
	mode: String, // "workspace", "relative", "togglefloating", "kill", "fullscreen"
	value: String, // workspace number or +/- for relative
}

async fn hyprctl(args: &[&str]) -> Result<String, anyhow::Error> {
	let output = std::process::Command::new("hyprctl")
		.args(args)
		.output()?;
	Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn run_hyprland_action(mode: &str, value: &str) -> Result<(), anyhow::Error> {
	match mode {
		"workspace" => {
			hyprctl(&["dispatch", "workspace", value]).await?;
		}
		"relative" => {
			if value == "+" {
				hyprctl(&["dispatch", "workspace", "+1"]).await?;
			} else if value == "-" {
				hyprctl(&["dispatch", "workspace", "-1"]).await?;
			}
		}
		"togglefloating" => {
			hyprctl(&["dispatch", "togglefloating", ""]).await?;
		}
		"kill" => {
			hyprctl(&["dispatch", "killactive", ""]).await?;
		}
		"fullscreen" => {
			hyprctl(&["dispatch", "fullscreen", "1"]).await?;
		}
		"togglepin" => {
			hyprctl(&["dispatch", "pin", ""]).await?;
		}
		_ => {}
	}
	Ok(())
}

pub struct HyprlandWorkspaceAction;
#[async_trait]
impl Action for HyprlandWorkspaceAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.hyprlandworkspace";
	type Settings = HyprlandWorkspaceSettings;

	async fn key_down(
		&self,
		_instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let mode = settings.mode.clone();
		let value = settings.value.clone();
		tokio::spawn(async move {
			if let Err(error) = run_hyprland_action(&mode, &value).await {
				log::warn!("Failed to run hyprland action: {error}");
			}
		});
		Ok(())
	}

	async fn dial_rotate(
		&self,
		_instance: &Instance,
		_settings: &Self::Settings,
		ticks: i16,
		_pressed: bool,
	) -> OpenActionResult<()> {
		let direction = if ticks > 0 { "+" } else { "-" };
		tokio::spawn(async move {
			if let Err(error) = run_hyprland_action("relative", direction).await {
				log::warn!("Failed to rotate workspace: {error}");
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
