use std::collections::HashMap;
use std::sync::LazyLock;

use image::DynamicImage;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::Duration;

use crate::shared::Context;

/// Eine extrahierte GIF-Animation
struct GifAnimation {
	frames: Vec<DynamicImage>,
	delays: Vec<Duration>,
	loop_count: Option<u16>,
}

/// Globaler Registry für laufende Animationen
static ACTIVE_ANIMATIONS: LazyLock<RwLock<HashMap<String, JoinHandle<()>>>> =
	LazyLock::new(|| RwLock::new(HashMap::new()));

/// Extrahiert Frames aus GIF-Daten
fn extract_gif_frames(bytes: &[u8]) -> Result<GifAnimation, anyhow::Error> {
	let mut options = gif::DecodeOptions::new();
	options.set_color_output(gif::ColorOutput::RGBA);
	let mut decoder = options.read_info(bytes)?;

	let mut frames = Vec::new();
	let mut delays = Vec::new();
	let loop_count = decoder.loop_count();

	while let Some(frame) = decoder.read_next_frame()? {
		let image = image::RgbaImage::from_raw(
			frame.width as u32,
			frame.height as u32,
			frame.buffer.to_vec(),
		)
		.ok_or_else(|| anyhow::anyhow!("Invalid frame dimensions"))?;

		frames.push(DynamicImage::ImageRgba8(image));
		// GIF delay is in 1/100ths of a second, 0 means ~10ms (1/100s min)
		let delay_ms = (frame.delay as u64).max(1) * 10;
		delays.push(Duration::from_millis(delay_ms));
	}

	Ok(GifAnimation { frames, delays, loop_count })
}

fn context_key(context: &Context) -> String {
	format!("{}:{}:{}:{}", context.device, context.profile, context.controller, context.position)
}

/// Startet oder ersetzt eine Animation für einen Context
pub async fn start_animation(context: Context, gif_bytes: Vec<u8>) -> Result<(), anyhow::Error> {
	// Alte Animation stoppen
	stop_animation(&context).await;

	let animation = extract_gif_frames(&gif_bytes)?;
	if animation.frames.is_empty() {
		return Ok(());
	}

	let key = context_key(&context);

	let handle = tokio::spawn(async move {
		let mut frame_index = 0usize;
		let mut loop_counter = 0u16;

		loop {
			let frame = &animation.frames[frame_index];
			let delay = animation.delays.get(frame_index).copied().unwrap_or(Duration::from_millis(100));

			// Frame als PNG Base64 konvertieren und an elgato senden
			let mut buffer = Vec::new();
			if frame.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png).is_ok() {
				let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
				let data_url = format!("data:image/png;base64,{}", base64);
				let _ = crate::elgato::update_image(&context, Some(&data_url)).await;
			}

			tokio::time::sleep(delay).await;

			frame_index += 1;
			if frame_index >= animation.frames.len() {
				frame_index = 0;
				loop_counter += 1;

				// Prüfe Loop-Count (None = infinite)
				if let Some(max_loops) = animation.loop_count {
					if loop_counter >= max_loops {
						break;
					}
				}
			}
		}
	});

	ACTIVE_ANIMATIONS.write().await.insert(key, handle);
	Ok(())
}

/// Stoppt eine laufende Animation
pub async fn stop_animation(context: &Context) {
	let key = context_key(context);
	if let Some((_, handle)) = ACTIVE_ANIMATIONS.write().await.remove(&key) {
		handle.abort();
	}
}
