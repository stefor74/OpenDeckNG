use openaction::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct NotificationSettings {
	title: String,
	body: String,
	urgency: String, // low, normal, critical
	icon: String,
}

async fn send_notification(
	title: &str,
	body: &str,
	urgency: &str,
	icon: &str,
) -> Result<(), anyhow::Error> {
	let urgency_arg = match urgency {
		"low" => "low",
		"critical" => "critical",
		_ => "normal",
	};

	let mut args = vec![
		"--urgency", urgency_arg,
	];

	if !icon.is_empty() {
		args.push("--icon");
		args.push(icon);
	}

	args.push(title);
	if !body.is_empty() {
		args.push(body);
	}

	std::process::Command::new("notify-send")
		.args(&args)
		.output()?;

	Ok(())
}

pub struct NotificationAction;
#[async_trait]
impl Action for NotificationAction {
	const UUID: &'static str = "com.stefor74.opendeckng.linux.notification";
	type Settings = NotificationSettings;

	async fn key_down(
		&self,
		_instance: &Instance,
		settings: &Self::Settings,
	) -> OpenActionResult<()> {
		let title = settings.title.clone();
		let body = settings.body.clone();
		let urgency = settings.urgency.clone();
		let icon = settings.icon.clone();

		tokio::spawn(async move {
			if let Err(error) = send_notification(&title, &body, &urgency, &icon).await {
				log::warn!("Failed to send notification: {error}");
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
