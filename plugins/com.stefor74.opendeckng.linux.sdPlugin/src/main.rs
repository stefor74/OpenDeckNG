mod counter;
mod hyprland;
mod media;
mod notification;
mod system_monitor;
mod volume;

use openaction::*;

struct GlobalEventHandler;
#[async_trait]
impl global_events::GlobalEventHandler for GlobalEventHandler {}

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {}", error);
		}
	}

	global_events::set_global_event_handler(&GlobalEventHandler);
	register_action(media::MediaControlAction).await;
	register_action(volume::AudioVolumeAction).await;
	register_action(system_monitor::SystemMonitorAction).await;
	register_action(hyprland::HyprlandWorkspaceAction).await;
	register_action(counter::CounterAction).await;
	register_action(notification::NotificationAction).await;

	run(std::env::args().collect()).await
}
