#!/usr/bin/env python3
"""Generate OpenDeckNG icons in all required formats."""

from PIL import Image, ImageDraw, ImageFont
import struct
import io
import os

BASE_SIZE = 512

def create_base_icon(size: int = BASE_SIZE) -> Image.Image:
    """Create the base 512x512 icon."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Background: dark rounded square (like the app theme)
    bg_color = (23, 23, 23, 255)  # neutral-900
    accent_color = (34, 197, 94, 255)  # green-500
    button_colors = [
        (64, 64, 64, 255),
        (82, 82, 82, 255),
        (64, 64, 64, 255),
        (82, 82, 82, 255),
        (64, 64, 64, 255),
    ]

    # Corner radius
    radius = size // 8

    # Draw rounded background
    def rounded_rect(draw, xy, radius, fill):
        x1, y1, x2, y2 = xy
        draw.rounded_rectangle(xy, radius=radius, fill=fill)

    rounded_rect(draw, (0, 0, size, size), radius, bg_color)

    # Draw a 3x2 grid of "buttons" (like a Stream Deck)
    grid_cols = 3
    grid_rows = 2
    padding = size // 12
    gap = size // 24
    cell_w = (size - 2 * padding - (grid_cols - 1) * gap) // grid_cols
    cell_h = (size - 2 * padding - (grid_rows - 1) * gap) // grid_rows
    cell_radius = cell_w // 8

    for row in range(grid_rows):
        for col in range(grid_cols):
            idx = row * grid_cols + col
            x = padding + col * (cell_w + gap)
            y = padding + row * (cell_h + gap)
            # Make the last cell the accent (green) one
            if idx == 5:
                color = accent_color
            else:
                color = button_colors[idx % len(button_colors)]
            draw.rounded_rectangle((x, y, x + cell_w, y + cell_h), radius=cell_radius, fill=color)

    # Draw "NG" text at bottom center
    try:
        font_size = size // 6
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", font_size)
    except:
        font = ImageFont.load_default()

    text = "NG"
    bbox = draw.textbbox((0, 0), text, font=font)
    text_w = bbox[2] - bbox[0]
    text_h = bbox[3] - bbox[1]
    text_x = (size - text_w) // 2
    text_y = size - padding - text_h - size // 48

    # Text shadow
    draw.text((text_x + 2, text_y + 2), text, font=font, fill=(0, 0, 0, 180))
    # Text
    draw.text((text_x, text_y), text, font=font, fill=(255, 255, 255, 255))

    return img


def create_ico(png_path: str, ico_path: str):
    """Create Windows .ico with multiple resolutions."""
    img = Image.open(png_path)
    sizes = [16, 32, 48, 128, 256]
    imgs = []
    for s in sizes:
        imgs.append(img.resize((s, s), Image.Resampling.LANCZOS))
    imgs[0].save(ico_path, format="ICO", sizes=[(s, s) for s in sizes])


def create_icns(png_path: str, icns_path: str):
    """Create macOS .icns (simple implementation)."""
    # icns format is complex; we'll generate multiple PNGs inside a resource fork-like structure
    # or use a simpler approach: just copy the PNG for now and note that real icns needs icnsutil
    # Actually let's build a minimal valid icns file
    img = Image.open(png_path)
    sizes = [
        (16, b"icp4"),
        (32, b"icp5"),
        (64, b"icp6"),
        (128, b"ic07"),
        (256, b"ic08"),
        (512, b"ic09"),
    ]

    # Build ICNS file manually
    # Header: 'icns' + 4-byte file length
    # Each entry: 4-byte type + 4-byte length + PNG data
    entries = []
    for px, type_code in sizes:
        resized = img.resize((px, px), Image.Resampling.LANCZOS)
        buf = io.BytesIO()
        resized.save(buf, format="PNG")
        data = buf.getvalue()
        length = 8 + len(data)
        entries.append(struct.pack(">4sI", type_code, length) + data)

    body = b"".join(entries)
    file_length = 8 + len(body)
    header = struct.pack(">4sI", b"icns", file_length)

    with open(icns_path, "wb") as f:
        f.write(header + body)


def create_plugin_icon(size: int = 128) -> Image.Image:
    """Create a smaller icon for plugins."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    bg_color = (23, 23, 23, 255)
    accent_color = (34, 197, 94, 255)
    radius = size // 8

    draw.rounded_rectangle((0, 0, size, size), radius=radius, fill=bg_color)

    # Simple 2x2 grid
    padding = size // 8
    gap = size // 16
    cell = (size - 2 * padding - gap) // 2
    cell_r = cell // 6

    draw.rounded_rectangle((padding, padding, padding + cell, padding + cell), radius=cell_r, fill=(64, 64, 64, 255))
    draw.rounded_rectangle((padding + cell + gap, padding, padding + 2 * cell + gap, padding + cell), radius=cell_r, fill=(82, 82, 82, 255))
    draw.rounded_rectangle((padding, padding + cell + gap, padding + cell, padding + 2 * cell + gap), radius=cell_r, fill=(82, 82, 82, 255))
    draw.rounded_rectangle((padding + cell + gap, padding + cell + gap, padding + 2 * cell + gap, padding + 2 * cell + gap), radius=cell_r, fill=accent_color)

    return img


def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    icons_dir = os.path.join(base_dir, "src-tauri", "icons")
    plugins_dir = os.path.join(base_dir, "plugins")
    static_dir = os.path.join(base_dir, "static")

    os.makedirs(icons_dir, exist_ok=True)

    # Step 1: Base 512x512 PNG
    print("Step 1: Generating base icon (512x512)...")
    base = create_base_icon(512)
    base_path = os.path.join(icons_dir, "icon.png")
    base.save(base_path)
    print(f"  -> {base_path}")

    # Step 2: Windows ICO
    print("Step 2: Generating Windows icon.ico...")
    ico_path = os.path.join(icons_dir, "icon.ico")
    create_ico(base_path, ico_path)
    print(f"  -> {ico_path}")

    # Step 3: macOS ICNS
    print("Step 3: Generating macOS icon.icns...")
    icns_path = os.path.join(icons_dir, "icon.icns")
    create_icns(base_path, icns_path)
    print(f"  -> {icns_path}")

    # Step 4: Plugin icons
    print("Step 4: Generating plugin icons...")
    plugin_icon = create_plugin_icon(128)
    for plugin_name in ["com.amansprojects.starterpack.sdPlugin", "com.stefor74.opendeckng.linux.sdPlugin"]:
        plugin_icon_dir = os.path.join(plugins_dir, plugin_name, "assets", "icons")
        os.makedirs(plugin_icon_dir, exist_ok=True)
        p = os.path.join(plugin_icon_dir, "plugin.png")
        plugin_icon.save(p)
        print(f"  -> {p}")

    # Step 5: Static icons
    print("Step 5: Generating static icons...")
    static_icon = create_plugin_icon(64)
    for name in ["cube", "alert", "ok", "multi-action", "toggle-action"]:
        p = os.path.join(static_dir, f"{name}.png")
        static_icon.save(p)
        print(f"  -> {p}")

    print("\nAll icons generated successfully!")


if __name__ == "__main__":
    main()
