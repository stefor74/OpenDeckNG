use std::collections::HashMap;
use std::sync::LazyLock;

use base64::Engine as _;
use elgato_streamdeck::images::ImageRect;
use elgato_streamdeck::info::Kind;
use image::{DynamicImage, GenericImageView};
use tokio::sync::RwLock;

use crate::shared::Context;

/// Cached background image per device+profile
static BACKGROUND_CACHE: LazyLock<RwLock<HashMap<String, DynamicImage>>> =
	LazyLock::new(|| RwLock::new(HashMap::new()));

fn cache_key(device: &str, profile: &str) -> String {
	format!("{}:{}", device, profile)
}

/// Lädt und cached ein Hintergrundbild für ein Device+Profile
pub async fn load_background(device: &str, profile: &str, image_data: &str) -> Result<(), anyhow::Error> {
	let key = cache_key(device, profile);

	// Decode base64 data URL
	let bytes = if image_data.starts_with("data:") {
		let data = image_data.split_once(',').unwrap().1;
		base64::engine::general_purpose::STANDARD.decode(data)?
	} else {
		std::fs::read(image_data)?
	};

	let img = image::load_from_memory(&bytes)?;
	BACKGROUND_CACHE.write().await.insert(key, img);
	Ok(())
}

/// Entfernt einen Hintergrund aus dem Cache
pub async fn clear_background(device: &str, profile: &str) {
	BACKGROUND_CACHE.write().await.remove(&cache_key(device, profile));
}

/// Rendert den Hintergrund auf das Device (nur LCD-Strip für Plus/Neo)
pub async fn render_background(device_id: &str, profile: &str) -> Result<(), anyhow::Error> {
	use crate::elgato::ELGATO_DEVICES;

	let key = cache_key(device_id, profile);
	let cache = BACKGROUND_CACHE.read().await;
	let Some(bg) = cache.get(&key) else {
		return Ok(());
	};
	let bg = bg.clone();
	drop(cache);

	let devices = ELGATO_DEVICES.read().await;
	let Some(device) = devices.get(device_id) else {
		return Ok(());
	};

	let kind = device.kind();

	// Nur für Devices mit LCD-Strip
	let Some(lcd_format) = kind.lcd_image_format() else {
		return Ok(());
	};

	// LCD-Strip sizes per device kind
	let (lcd_w, lcd_h): (u32, u32) = match kind {
		Kind::Plus => (800, 100),
		_ => return Ok(()),
	};

	// Skaliere Hintergrund auf LCD-Größe
	let scaled = bg.resize_exact(lcd_w as u32, lcd_h as u32, image::imageops::FilterType::Lanczos3);

	// Schreibe auf LCD-Strip
	let rect = ImageRect::from_image_async(
		elgato_streamdeck::images::convert_image_with_format_async(lcd_format, scaled)?
	)?;
	device.write_lcd_fill(&rect).await?;
	device.flush().await?;

	Ok(())
}

/// Rendert einen einzelnen Button mit Hintergrund-Compositing
/// (für klassische Devices ohne LCD-Strip)
pub async fn composite_button_image(
	context: &Context,
	button_image: Option<&str>,
) -> Result<Option<String>, anyhow::Error> {
	use crate::shared::DEVICES;

	let device_info = DEVICES.get(&context.device);
	let Some(device_info) = device_info else {
		return Ok(button_image.map(|s| s.to_owned()));
	};

	let profile = &context.profile;
	let key = cache_key(&context.device, profile);
	let cache = BACKGROUND_CACHE.read().await;
	let Some(bg) = cache.get(&key) else {
		return Ok(button_image.map(|s| s.to_owned()));
	};
	let bg = bg.clone();
	drop(cache);

	// Button-Größe in Pixeln (Standard Elgato)
	let button_size: u32 = 72;

	// Berechne Grid-Position
	let cols = device_info.columns as u32;
	let rows = device_info.rows as u32;
	let position = context.position as u32;

	if cols == 0 || rows == 0 || position >= cols * rows {
		return Ok(button_image.map(|s| s.to_owned()));
	}

	let col = position % cols;
	let row = position / cols;

	// Skaliere Hintergrund auf Device-Größe (cols * button_size, rows * button_size)
	let device_w = cols * button_size;
	let device_h = rows * button_size;
	let scaled_bg = bg.resize_exact(device_w, device_h, image::imageops::FilterType::Lanczos3);

	// Extrahiere Button-Fragment
	let x = col * button_size;
	let y = row * button_size;
	let bg_fragment = scaled_bg.view(x, y, button_size, button_size).to_image();

	// Wenn kein Button-Bild, gib nur das Fragment zurück
	let Some(button_img_str) = button_image else {
		let mut buffer = Vec::new();
		DynamicImage::ImageRgba8(bg_fragment).write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
		let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
		return Ok(Some(format!("data:image/png;base64,{}", base64)));
	};

	// Lade Button-Bild
	let button_bytes = if button_img_str.starts_with("data:") {
		let data = button_img_str.split_once(',').unwrap().1;
		base64::engine::general_purpose::STANDARD.decode(data)?
	} else {
		return Ok(Some(button_img_str.to_owned()));
	};

	let button_img = image::load_from_memory(&button_bytes)?;
	let button_img = button_img.resize_exact(button_size, button_size, image::imageops::FilterType::Lanczos3);

	// Composite: Hintergrund + Button-Bild (alpha blending)
	let mut composite = bg_fragment.clone();
	for (x, y, pixel) in button_img.to_rgba8().enumerate_pixels() {
		let alpha = pixel[3] as f32 / 255.0;
		if alpha > 0.0 {
			let bg_pixel = composite.get_pixel_mut(x, y);
			bg_pixel[0] = ((pixel[0] as f32 * alpha) + (bg_pixel[0] as f32 * (1.0 - alpha))) as u8;
			bg_pixel[1] = ((pixel[1] as f32 * alpha) + (bg_pixel[1] as f32 * (1.0 - alpha))) as u8;
			bg_pixel[2] = ((pixel[2] as f32 * alpha) + (bg_pixel[2] as f32 * (1.0 - alpha))) as u8;
			bg_pixel[3] = 255;
		}
	}

	let mut buffer = Vec::new();
	DynamicImage::ImageRgba8(composite).write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
	let base64 = base64::engine::general_purpose::STANDARD.encode(&buffer);
	Ok(Some(format!("data:image/png;base64,{}", base64)))
}
