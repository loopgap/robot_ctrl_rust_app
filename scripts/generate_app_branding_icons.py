import os
import subprocess
import tempfile
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
BRANDING_DIR = os.path.join(ROOT_DIR, "assets", "branding")
os.makedirs(BRANDING_DIR, exist_ok=True)

EDGE_PATH = r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"

# ── 1. ROBOT CONTROL SUITE APP SVG (256x256) ──
# Futuristic Industrial Robotics: Deep cyber navy squircle, robotic servo core,
# dual articulated precision gripper arms, trajectory guidance crosshair,
# telemetry orbit rings, neon cyan / electric blue gradients.
ROBOT_CONTROL_SVG = """<svg width="256" height="256" viewBox="0 0 256 256" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <!-- Background Gradient -->
    <linearGradient id="rc-bg" x1="28" y1="20" x2="228" y2="236" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#0E192D"/>
      <stop offset="50%" stop-color="#0A1222"/>
      <stop offset="100%" stop-color="#050913"/>
    </linearGradient>

    <!-- Outer Border Glow Gradient -->
    <linearGradient id="rc-border" x1="32" y1="24" x2="224" y2="232" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#00F0FF" stop-opacity="0.85"/>
      <stop offset="35%" stop-color="#0284C7" stop-opacity="0.45"/>
      <stop offset="70%" stop-color="#1E3A8A" stop-opacity="0.25"/>
      <stop offset="100%" stop-color="#38BDF8" stop-opacity="0.6"/>
    </linearGradient>

    <!-- Primary Cyan-Blue Gradient -->
    <linearGradient id="rc-cyan" x1="60" y1="60" x2="196" y2="196" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#38BDF8"/>
      <stop offset="40%" stop-color="#00E5FF"/>
      <stop offset="100%" stop-color="#0284C7"/>
    </linearGradient>

    <!-- Accent Neon Cyan Gradient -->
    <linearGradient id="rc-neon" x1="128" y1="70" x2="128" y2="186" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#A5F3FC"/>
      <stop offset="50%" stop-color="#00F0FF"/>
      <stop offset="100%" stop-color="#0284C7"/>
    </linearGradient>

    <!-- Inner Core Glow -->
    <radialGradient id="rc-core-glow" cx="128" cy="128" r="64" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#00F0FF" stop-opacity="0.45"/>
      <stop offset="60%" stop-color="#0284C7" stop-opacity="0.12"/>
      <stop offset="100%" stop-color="#000000" stop-opacity="0"/>
    </radialGradient>

    <!-- Glass Reflection Highlight -->
    <linearGradient id="rc-glass" x1="128" y1="18" x2="128" y2="110" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.18"/>
      <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.0"/>
    </linearGradient>

    <!-- Drop Shadow Filter for central elements -->
    <filter id="rc-shadow" x="30" y="30" width="196" height="196" filterUnits="userSpaceOnUse">
      <feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#00E5FF" flood-opacity="0.32"/>
    </filter>
  </defs>

  <!-- Base Squircle Container (16px inset, 52px radius) -->
  <rect x="18" y="18" width="220" height="220" rx="52" fill="url(#rc-bg)"/>
  <rect x="18" y="18" width="220" height="220" rx="52" stroke="url(#rc-border)" stroke-width="2.5"/>

  <!-- Top Glass Highlight Overlay -->
  <path d="M 22 70 C 22 41.28 45.28 18 74 18 L 182 18 C 210.72 18 234 41.28 234 70 C 234 94 186 112 128 112 C 70 112 22 94 22 70 Z" fill="url(#rc-glass)"/>

  <!-- Telemetry Radar / Calibration Grid Rings (subtle background) -->
  <circle cx="128" cy="128" r="76" stroke="#0284C7" stroke-width="1.2" stroke-opacity="0.16" stroke-dasharray="4 6"/>
  <circle cx="128" cy="128" r="54" stroke="#00F0FF" stroke-width="1.2" stroke-opacity="0.22"/>
  <circle cx="128" cy="128" r="32" stroke="#38BDF8" stroke-width="1.0" stroke-opacity="0.20" stroke-dasharray="2 4"/>

  <!-- Central Radial Core Glow -->
  <circle cx="128" cy="128" r="64" fill="url(#rc-core-glow)"/>

  <!-- Precision Robotic Foreground Device (Group with glow shadow) -->
  <g filter="url(#rc-shadow)">
    <!-- Base Mounting Pedestal (Bottom) -->
    <path d="M 88 188 L 168 188 L 160 198 L 96 198 Z" fill="#0284C7" fill-opacity="0.4"/>
    <path d="M 96 198 L 160 198 L 154 205 L 102 205 Z" fill="#00E5FF" fill-opacity="0.6"/>

    <!-- Dual Lower Articulated Arms (Joint Beams) -->
    <path d="M 128 174 L 84 146 L 76 156 L 118 184 Z" fill="url(#rc-cyan)" fill-opacity="0.9"/>
    <path d="M 128 174 L 172 146 L 180 156 L 138 184 Z" fill="url(#rc-cyan)" fill-opacity="0.9"/>

    <!-- Left & Right Joint Actuator Hubs -->
    <circle cx="78" cy="150" r="10" fill="#0B132B" stroke="#00F0FF" stroke-width="2.5"/>
    <circle cx="78" cy="150" r="4" fill="#38BDF8"/>
    <circle cx="178" cy="150" r="10" fill="#0B132B" stroke="#00F0FF" stroke-width="2.5"/>
    <circle cx="178" cy="150" r="4" fill="#38BDF8"/>

    <!-- Upper Servo Manipulator Linkages -->
    <path d="M 78 144 L 88 92 L 100 95 L 86 146 Z" fill="url(#rc-cyan)"/>
    <path d="M 178 144 L 168 92 L 156 95 L 170 146 Z" fill="url(#rc-cyan)"/>

    <!-- Articulated Gripper End-Effectors (Left & Right Claws) -->
    <path d="M 88 92 C 88 74 104 60 118 56 L 114 68 C 104 72 96 82 96 92 Z" fill="url(#rc-neon)"/>
    <path d="M 168 92 C 168 74 152 60 138 56 L 142 68 C 152 72 160 82 160 92 Z" fill="url(#rc-neon)"/>

    <!-- Central Precision Sensor / Trajectory Crosshair Unit -->
    <!-- Hexagonal Servo Chassis Body -->
    <polygon points="128,96 156,112 156,144 128,160 100,144 100,112" fill="#071326" stroke="#00F0FF" stroke-width="2.6" stroke-linejoin="round"/>

    <!-- Central Laser / Gyro Core -->
    <circle cx="128" cy="128" r="16" fill="#0B2545" stroke="#38BDF8" stroke-width="2"/>
    <circle cx="128" cy="128" r="8" fill="#A5F3FC"/>
    <circle cx="128" cy="128" r="3.5" fill="#FFFFFF"/>

    <!-- Sub-Millimeter Optical Crosshair Reticle -->
    <line x1="128" y1="102" x2="128" y2="114" stroke="#00F0FF" stroke-width="2.2" stroke-linecap="round"/>
    <line x1="128" y1="142" x2="128" y2="154" stroke="#00F0FF" stroke-width="2.2" stroke-linecap="round"/>
    <line x1="106" y1="128" x2="118" y2="128" stroke="#00F0FF" stroke-width="2.2" stroke-linecap="round"/>
    <line x1="138" y1="128" x2="150" y2="128" stroke="#00F0FF" stroke-width="2.2" stroke-linecap="round"/>

    <!-- Dynamic Motion Vector Arcs (Closed Loop Feedback indicator) -->
    <path d="M 64 116 C 58 132 60 150 70 166" stroke="#00F0FF" stroke-width="2.4" stroke-linecap="round"/>
    <path d="M 192 116 C 198 132 196 150 186 166" stroke="#00F0FF" stroke-width="2.4" stroke-linecap="round"/>
    <path d="M 112 48 C 122 45 134 45 144 48" stroke="#38BDF8" stroke-width="2.2" stroke-linecap="round"/>
  </g>
</svg>
"""

# ── 2. RUST TOOLS SUITE APP SVG (256x256) ──
# Developer Multi-Tool Workshop: Dark obsidian squircle, iconic Rust gear matrix,
# precision tool caliper, code bracket { } core, glowing microchip pins,
# molten ember / radiant rust orange / golden amber gradients.
RUST_TOOLS_SVG = """<svg width="256" height="256" viewBox="0 0 256 256" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <!-- Background Gradient -->
    <linearGradient id="rt-bg" x1="28" y1="20" x2="228" y2="236" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#1A1817"/>
      <stop offset="50%" stop-color="#121110"/>
      <stop offset="100%" stop-color="#080807"/>
    </linearGradient>

    <!-- Outer Border Glow Gradient -->
    <linearGradient id="rt-border" x1="32" y1="24" x2="224" y2="232" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FF7A00" stop-opacity="0.9"/>
      <stop offset="35%" stop-color="#EA580C" stop-opacity="0.5"/>
      <stop offset="70%" stop-color="#7C2D12" stop-opacity="0.3"/>
      <stop offset="100%" stop-color="#FBBF24" stop-opacity="0.7"/>
    </linearGradient>

    <!-- Fiery Rust Amber-Orange Gradient -->
    <linearGradient id="rt-ember" x1="50" y1="50" x2="206" y2="206" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FCD34D"/>
      <stop offset="30%" stop-color="#F59E0B"/>
      <stop offset="70%" stop-color="#EA580C"/>
      <stop offset="100%" stop-color="#B45309"/>
    </linearGradient>

    <!-- Radiant Gold Highlight Gradient -->
    <linearGradient id="rt-gold" x1="128" y1="60" x2="128" y2="196" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FEF08A"/>
      <stop offset="50%" stop-color="#F59E0B"/>
      <stop offset="100%" stop-color="#DC2626"/>
    </linearGradient>

    <!-- Inner Core Warm Glow -->
    <radialGradient id="rt-core-glow" cx="128" cy="128" r="68" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#F97316" stop-opacity="0.45"/>
      <stop offset="60%" stop-color="#B45309" stop-opacity="0.14"/>
      <stop offset="100%" stop-color="#000000" stop-opacity="0"/>
    </radialGradient>

    <!-- Glass Reflection Highlight -->
    <linearGradient id="rt-glass" x1="128" y1="18" x2="128" y2="110" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.16"/>
      <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0.0"/>
    </linearGradient>

    <!-- Drop Shadow Filter for gear/tools -->
    <filter id="rt-shadow" x="30" y="30" width="196" height="196" filterUnits="userSpaceOnUse">
      <feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#EA580C" flood-opacity="0.38"/>
    </filter>
  </defs>

  <!-- Base Squircle Container (16px inset, 52px radius) -->
  <rect x="18" y="18" width="220" height="220" rx="52" fill="url(#rt-bg)"/>
  <rect x="18" y="18" width="220" height="220" rx="52" stroke="url(#rt-border)" stroke-width="2.5"/>

  <!-- Top Glass Highlight Overlay -->
  <path d="M 22 70 C 22 41.28 45.28 18 74 18 L 182 18 C 210.72 18 234 41.28 234 70 C 234 94 186 112 128 112 C 70 112 22 94 22 70 Z" fill="url(#rt-glass)"/>

  <!-- Subtle Diagnostic Hexagonal Honeycomb Grid -->
  <polygon points="128,48 162,68 162,108 128,128 94,108 94,68" stroke="#F59E0B" stroke-width="1.0" stroke-opacity="0.12" fill="none"/>
  <polygon points="128,128 162,148 162,188 128,208 94,188 94,148" stroke="#F59E0B" stroke-width="1.0" stroke-opacity="0.12" fill="none"/>
  <circle cx="128" cy="128" r="78" stroke="#EA580C" stroke-width="1.2" stroke-opacity="0.18" stroke-dasharray="6 8"/>

  <!-- Warm Core Radiant Glow -->
  <circle cx="128" cy="128" r="68" fill="url(#rt-core-glow)"/>

  <!-- Central Rust Gear & Multi-Tool Matrix Symbol -->
  <g filter="url(#rt-shadow)">
    <!-- Outer 8-Teeth Industrial Cogwheel (Precision Cut) -->
    <!-- Rotated 8 gear teeth -->
    <path d="
      M 120 54 L 136 54 L 138 68 C 146 71 154 75 161 81 L 172 73 L 183 84 L 175 95 C 181 102 185 110 188 118 L 202 120 L 202 136 L 188 138 C 185 146 181 154 175 161 L 183 172 L 172 183 L 161 175 C 154 181 146 185 138 188 L 136 202 L 120 202 L 118 188 C 110 185 102 181 95 175 L 84 183 L 73 172 L 81 161 C 75 154 71 146 68 138 L 54 136 L 54 120 L 68 118 C 71 110 75 102 81 95 L 73 84 L 84 73 L 95 81 C 102 75 110 71 118 68 Z
    " fill="url(#rt-ember)" stroke="#FEF08A" stroke-width="2.0" stroke-linejoin="round"/>

    <!-- Inner Metallic Core Disc -->
    <circle cx="128" cy="128" r="48" fill="#181615" stroke="url(#rt-gold)" stroke-width="3"/>
    <circle cx="128" cy="128" r="42" fill="#0F0E0D"/>

    <!-- Microprocessor Chip Traces / Bus Pins (4 directions) -->
    <line x1="128" y1="88" x2="128" y2="98" stroke="#F59E0B" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="128" y1="158" x2="128" y2="168" stroke="#F59E0B" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="88" y1="128" x2="98" y2="128" stroke="#F59E0B" stroke-width="2.5" stroke-linecap="round"/>
    <line x1="158" y1="128" x2="168" y2="128" stroke="#F59E0B" stroke-width="2.5" stroke-linecap="round"/>

    <!-- Code Brackets { } Sculpture in Center (The Spirit of Rust Craftsmanship) -->
    <!-- Left Bracket '{' -->
    <path d="M 116 106 C 111 106 107 109 107 114 L 107 122 C 107 125 104 128 100 128 C 104 128 107 131 107 134 L 107 142 C 107 147 111 150 116 150" stroke="#FCD34D" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/>

    <!-- Right Bracket '}' -->
    <path d="M 140 106 C 145 106 149 109 149 114 L 149 122 C 149 125 152 128 156 128 C 152 128 149 131 149 134 L 149 142 C 149 147 145 150 140 150" stroke="#FCD34D" stroke-width="3.5" stroke-linecap="round" stroke-linejoin="round"/>

    <!-- Central Micro-Core Node (Electric Cyan to highlight high-frequency data inspector) -->
    <circle cx="128" cy="128" r="5.5" fill="#38BDF8" stroke="#FFFFFF" stroke-width="1.8"/>

    <!-- Top & Bottom Tool Caliper Accent Notches -->
    <polygon points="128,74 133,82 123,82" fill="#FEF08A"/>
    <polygon points="128,182 133,174 123,174" fill="#FEF08A"/>
  </g>
</svg>
"""

def render_svg_to_png(svg_content, out_png_path, size=512):
    tmp_html = os.path.join(tempfile.gettempdir(), f"render_{os.path.basename(out_png_path)}.html")
    html_content = f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{ width: {size}px; height: {size}px; background: transparent; overflow: hidden; }}
  svg {{ width: {size}px; height: {size}px; display: block; }}
</style>
</head>
<body>
{svg_content}
</body>
</html>"""
    with open(tmp_html, "w", encoding="utf-8") as f:
        f.write(html_content)

    cmd = [
        EDGE_PATH,
        "--headless",
        "--disable-gpu",
        "--force-device-scale-factor=1",
        "--default-background-color=00000000",
        f"--screenshot={out_png_path}",
        f"--window-size={size},{size}",
        f"file:///{tmp_html.replace(chr(92), '/')}",
    ]
    subprocess.run(cmd, check=True)
    if os.path.exists(tmp_html):
        os.remove(tmp_html)

def generate_app_icon_package(name, svg_content):
    print(f"=== Generating Branding Package for: {name} ===")
    
    # 1. Write SVG
    svg_path = os.path.join(BRANDING_DIR, f"{name}.svg")
    with open(svg_path, "w", encoding="utf-8") as f:
        f.write(svg_content.strip() + "\n")
    print(f"  -> SVG written: {svg_path}")

    # 2. Render 512x512 Master PNG
    master_png = os.path.join(BRANDING_DIR, f"{name}_512.png")
    render_svg_to_png(svg_content, master_png, size=512)
    print(f"  -> Master PNG rendered: {master_png}")

    # 3. Create 256x256 and 64x64 PNGs with Pillow Lanczos
    master_img = Image.open(master_png)
    
    png_256_path = os.path.join(BRANDING_DIR, f"{name}_256.png")
    img_256 = master_img.resize((256, 256), Image.Resampling.LANCZOS)
    img_256.save(png_256_path, format="PNG")
    print(f"  -> 256x256 PNG saved: {png_256_path}")

    png_64_path = os.path.join(BRANDING_DIR, f"{name}_64.png")
    img_64 = master_img.resize((64, 64), Image.Resampling.LANCZOS)
    img_64.save(png_64_path, format="PNG")
    print(f"  -> 64x64 PNG saved: {png_64_path}")

    # 4. Generate multi-resolution .ico (16, 24, 32, 48, 64, 128, 256)
    ico_path = os.path.join(BRANDING_DIR, f"{name}.ico")
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    master_img.save(ico_path, format="ICO", sizes=ico_sizes)
    print(f"  -> Multi-resolution ICO saved: {ico_path}")

    master_img.close()
    # Remove 512 master to keep directory clean
    if os.path.exists(master_png):
        os.remove(master_png)

if __name__ == "__main__":
    generate_app_icon_package("robot_control_app", ROBOT_CONTROL_SVG)
    generate_app_icon_package("rust_tools_suite_app", RUST_TOOLS_SVG)
    print("\n[SUCCESS] All app branding icons generated successfully!")
