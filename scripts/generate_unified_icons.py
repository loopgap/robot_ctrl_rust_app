#!/usr/bin/env python3
"""
generate_unified_icons.py
Generates clean, standardized, production-grade SVG icons for:
- robot_control_rust (Robot Control Suite)
- rust_tools_suite (Developer Tools Suite)
- shared common actions

Design standards:
- 24x24 viewBox
- stroke="currentColor"
- stroke-width="1.8"
- stroke-linecap="round"
- stroke-linejoin="round"
- fill="none" (or fill="currentColor" for specific accents)
"""

import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent / "assets" / "icons"

SVG_HEADER = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">\n'
SVG_FOOTER = '</svg>\n'

ROBOT_ICONS = {
    "dashboard.svg": """  <path d="M12 3a9 9 0 0 0-9 9c0 3.1 1.6 5.8 4 7.4" />
  <path d="M17 19.4A9 9 0 0 0 21 12a9 9 0 0 0-9-9" />
  <path d="M12 12l3.5-3.5" />
  <circle cx="12" cy="12" r="2" fill="currentColor" />
  <path d="M7 12h1" />
  <path d="M16 12h1" />
  <path d="M12 7V8" />
  <path d="M8.5 8.5l.7.7" />
  <path d="M15.5 8.5l-.7.7" />""",

    "connections.svg": """  <rect x="3" y="8" width="6" height="8" rx="2" />
  <path d="M9 10h3" />
  <path d="M9 14h3" />
  <rect x="15" y="6" width="6" height="12" rx="2" />
  <path d="M12 10h3" />
  <path d="M12 14h3" />
  <circle cx="6" cy="12" r="1" fill="currentColor" />
  <circle cx="18" cy="12" r="1.5" fill="currentColor" />""",

    "terminal.svg": """  <rect x="3" y="4" width="18" height="16" rx="3" />
  <path d="M7 9l4 3-4 3" />
  <path d="M13 15h4" />
  <line x1="3" y1="8" x2="21" y2="8" stroke-width="1.2" opacity="0.4" />""",

    "protocol.svg": """  <rect x="3" y="4" width="18" height="16" rx="2.5" />
  <path d="M3 9h18" stroke-width="1.4" />
  <path d="M9 9v11" stroke-width="1.4" />
  <path d="M15 9v11" stroke-width="1.4" />
  <circle cx="6" cy="6.5" r="1" fill="currentColor" />
  <path d="M11 6.5h7" stroke-width="1.2" opacity="0.6" />
  <path d="M5.5 13h1" />
  <path d="M5.5 16h1" />
  <path d="M11.5 14h1" />
  <path d="M17.5 14h1" />""",

    "topology.svg": """  <circle cx="12" cy="12" r="3" fill="currentColor" />
  <circle cx="5" cy="6" r="2" />
  <circle cx="19" cy="6" r="2" />
  <circle cx="5" cy="18" r="2" />
  <circle cx="19" cy="18" r="2" />
  <path d="M6.8 7.5L9.8 10.2" />
  <path d="M17.2 7.5L14.2 10.2" />
  <path d="M6.8 16.5L9.8 13.8" />
  <path d="M17.2 16.5L14.2 13.8" />""",

    "pid.svg": """  <path d="M3 20h18" stroke-width="1.5" />
  <path d="M3 4v16" stroke-width="1.5" />
  <path d="M3 11h18" stroke-dasharray="2 2" stroke-width="1.2" opacity="0.6" />
  <path d="M3 19c2-8 4-13 7-13s3 8 5 8 3-4 6-4" />""",

    "neural.svg": """  <circle cx="5" cy="6" r="2" />
  <circle cx="5" cy="12" r="2" />
  <circle cx="5" cy="18" r="2" />
  <circle cx="12" cy="8" r="2" fill="currentColor" />
  <circle cx="12" cy="16" r="2" fill="currentColor" />
  <circle cx="19" cy="12" r="2" />
  <path d="M7 6.5l3.2 1" stroke-width="1.2" opacity="0.6" />
  <path d="M7 7.5l3.2 7" stroke-width="1.2" opacity="0.6" />
  <path d="M7 11.5l3.2-2.5" stroke-width="1.2" opacity="0.6" />
  <path d="M7 12.5l3.2 2.5" stroke-width="1.2" opacity="0.6" />
  <path d="M7 16.5l3.2-7" stroke-width="1.2" opacity="0.6" />
  <path d="M7 17.5l3.2-1" stroke-width="1.2" opacity="0.6" />
  <path d="M14 8.8l3.2 2.4" stroke-width="1.2" opacity="0.6" />
  <path d="M14 15.2l3.2-2.4" stroke-width="1.2" opacity="0.6" />""",

    "visualization.svg": """  <path d="M3 20h18" stroke-width="1.5" />
  <path d="M3 4v16" stroke-width="1.5" />
  <path d="M4 16l4.5-6 4 4 4.5-8 3 4" />
  <circle cx="8.5" cy="10" r="1.5" fill="currentColor" />
  <circle cx="12.5" cy="14" r="1.5" fill="currentColor" />
  <circle cx="17" cy="6" r="1.5" fill="currentColor" />""",

    "simulation.svg": """  <circle cx="12" cy="12" r="2.5" fill="currentColor" />
  <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(-30 12 12)" stroke-width="1.5" />
  <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(30 12 12)" stroke-width="1.5" />
  <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(90 12 12)" stroke-width="1.5" opacity="0.5" />""",

    "modbus.svg": """  <rect x="4" y="4" width="16" height="6.5" rx="2" />
  <rect x="4" y="13.5" width="16" height="6.5" rx="2" />
  <path d="M8 7.25h8" stroke-width="1.2" />
  <path d="M8 16.75h8" stroke-width="1.2" />
  <path d="M10 10.5v3m-2-1.5l2-1.5 2 1.5" stroke-width="1.3" />
  <path d="M14 13.5v-3m-2 1.5l2 1.5 2-1.5" stroke-width="1.3" />""",

    "canopen.svg": """  <rect x="3" y="6" width="6" height="5" rx="1.5" />
  <rect x="15" y="6" width="6" height="5" rx="1.5" />
  <rect x="9" y="13" width="6" height="5" rx="1.5" />
  <path d="M6 11v5h3" stroke-width="1.4" />
  <path d="M18 11v5h-3" stroke-width="1.4" />
  <path d="M12 13v-3" stroke-width="1.4" />
  <path d="M6 8.5h.5" />
  <path d="M18 8.5h.5" />
  <path d="M12 15.5h.5" />""",

    "chassis_differential.svg": """  <rect x="6" y="5" width="12" height="14" rx="3" />
  <rect x="3" y="10" width="3" height="7" rx="1.2" fill="currentColor" />
  <rect x="18" y="10" width="3" height="7" rx="1.2" fill="currentColor" />
  <circle cx="12" cy="7.5" r="1.8" fill="currentColor" />
  <path d="M9 13h6" stroke-width="1.2" />""",

    "chassis_mecanum.svg": """  <rect x="6" y="5" width="12" height="14" rx="2" />
  <rect x="3" y="5.5" width="3" height="5" rx="1" />
  <rect x="18" y="5.5" width="3" height="5" rx="1" />
  <rect x="3" y="13.5" width="3" height="5" rx="1" />
  <rect x="18" y="13.5" width="3" height="5" rx="1" />
  <path d="M3.5 6.5l2 3M18.5 6.5l2 3M3.5 17.5l2-3M18.5 17.5l2-3" stroke-width="1" />""",

    "chassis_omni3.svg": """  <polygon points="12 4 4 18 20 18" stroke-width="1.5" />
  <rect x="10.5" y="2" width="3" height="4" rx="1" fill="currentColor" />
  <rect x="2" y="16" width="3.5" height="4" rx="1" transform="rotate(-30 3.75 18)" fill="currentColor" />
  <rect x="18.5" y="16" width="3.5" height="4" rx="1" transform="rotate(30 20.25 18)" fill="currentColor" />
  <circle cx="12" cy="13" r="2" />""",

    "chassis_omni4.svg": """  <rect x="8" y="8" width="8" height="8" rx="2" />
  <rect x="10.5" y="3" width="3" height="5" rx="1" fill="currentColor" />
  <rect x="10.5" y="16" width="3" height="5" rx="1" fill="currentColor" />
  <rect x="3" y="10.5" width="5" height="3" rx="1" fill="currentColor" />
  <rect x="16" y="10.5" width="5" height="3" rx="1" fill="currentColor" />
  <circle cx="12" cy="12" r="1.5" />""",

    "chassis_ackermann.svg": """  <rect x="7" y="5" width="10" height="14" rx="2" />
  <rect x="3.5" y="4.5" width="3" height="5.5" rx="1" transform="rotate(-15 5 7.25)" fill="currentColor" />
  <rect x="17.5" y="4.5" width="3" height="5.5" rx="1" transform="rotate(-15 19 7.25)" fill="currentColor" />
  <rect x="3" y="14" width="3" height="5" rx="1" fill="currentColor" />
  <rect x="18" y="14" width="3" height="5" rx="1" fill="currentColor" />
  <path d="M5 7.25h14" stroke-width="1.2" />
  <path d="M4.5 16.5h15" stroke-width="1.2" />""",

    "chassis_tracked.svg": """  <rect x="2" y="4" width="5" height="16" rx="2.5" />
  <rect x="17" y="4" width="5" height="16" rx="2.5" />
  <rect x="7" y="7" width="10" height="10" rx="2" />
  <circle cx="4.5" cy="7" r="1" fill="currentColor" />
  <circle cx="4.5" cy="12" r="1" fill="currentColor" />
  <circle cx="4.5" cy="17" r="1" fill="currentColor" />
  <circle cx="19.5" cy="7" r="1" fill="currentColor" />
  <circle cx="19.5" cy="12" r="1" fill="currentColor" />
  <circle cx="19.5" cy="17" r="1" fill="currentColor" />""",

    "arm_scara.svg": """  <rect x="4" y="17" width="6" height="4" rx="1" />
  <circle cx="7" cy="15" r="2" fill="currentColor" />
  <path d="M7 15l6-5" stroke-width="2" />
  <circle cx="13" cy="10" r="2" fill="currentColor" />
  <path d="M13 10l5 2" stroke-width="2" />
  <circle cx="18" cy="12" r="1.5" />
  <path d="M18 13.5v5.5" stroke-width="1.5" />
  <path d="M16.5 19h3" stroke-width="1.8" />""",

    "arm_six_dof.svg": """  <rect x="4" y="18" width="8" height="3" rx="1" />
  <circle cx="8" cy="17" r="2" fill="currentColor" />
  <path d="M8 17l2-6" stroke-width="2" />
  <circle cx="10" cy="11" r="1.8" fill="currentColor" />
  <path d="M10 11l5-3" stroke-width="2" />
  <circle cx="15" cy="8" r="1.6" fill="currentColor" />
  <path d="M15 8l3 2" stroke-width="1.8" />
  <circle cx="18" cy="10" r="1.4" />
  <path d="M18 10l2 2" stroke-width="1.5" />
  <path d="M19 13l2-1M21 14l-1-2" stroke-width="1.5" />""",

    "robot_delta.svg": """  <path d="M5 5h14" stroke-width="2" />
  <circle cx="6" cy="5" r="1.5" fill="currentColor" />
  <circle cx="12" cy="5" r="1.5" fill="currentColor" />
  <circle cx="18" cy="5" r="1.5" fill="currentColor" />
  <path d="M6 6.5l3.5 5.5-1 5" stroke-width="1.4" />
  <path d="M12 6.5l-1 5.5 1 5" stroke-width="1.4" />
  <path d="M18 6.5l-3.5 5.5 1 5" stroke-width="1.4" />
  <path d="M8.5 17h7" stroke-width="2" />
  <path d="M12 17v3" stroke-width="1.8" />""",

    "generic.svg": """  <rect x="4" y="4" width="16" height="16" rx="3" stroke-width="1.5" />
  <path d="M8 12h8" stroke-width="1.4" />
  <path d="M12 8v8" stroke-width="1.4" />
  <circle cx="12" cy="12" r="2" fill="currentColor" />"""
}

TOOL_ICONS = {
    "at32_boot.svg": """  <rect x="5" y="5" width="14" height="14" rx="2.5" />
  <path d="M8 2v3M12 2v3M16 2v3" stroke-width="1.4" />
  <path d="M8 19v3M12 19v3M16 19v3" stroke-width="1.4" />
  <path d="M2 8h3M2 12h3M2 16h3" stroke-width="1.4" />
  <path d="M19 8h3M19 12h3M19 16h3" stroke-width="1.4" />
  <path d="M12 9v6m-2.5-2.5l2.5 2.5 2.5-2.5" stroke-width="1.6" />""",

    "checksum.svg": """  <path d="M12 3L4 6.5v5.5c0 5 3.5 9.5 8 10.5 4.5-1 8-5.5 8-10.5V6.5L12 3z" />
  <path d="M9 12l2 2 4-4" stroke-width="2" />""",

    "json_workshop.svg": """  <path d="M8 4c-2 0-3 1-3 3v3c0 1.5-1 2-2 2 1 0 2 .5 2 2v3c0 2 1 3 3 3" />
  <path d="M16 4c2 0 3 1 3 3v3c0 1.5 1 2 2 2-1 0-2 .5-2 2v3c0 2-1 3-3 3" />
  <circle cx="10" cy="12" r="1" fill="currentColor" />
  <circle cx="14" cy="12" r="1" fill="currentColor" />""",

    "log_inspector.svg": """  <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h7" />
  <path d="M14 3v5h5" />
  <path d="M8 10h4" />
  <path d="M8 14h3" />
  <circle cx="16.5" cy="16.5" r="3" />
  <path d="M18.8 18.8L21.5 21.5" stroke-width="2" />""",

    "url_codec.svg": """  <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
  <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
  <path d="M16 16l3 3" stroke-width="1.2" opacity="0.6" />""",

    "time_converter.svg": """  <circle cx="12" cy="12" r="8.5" />
  <path d="M12 7v5l3.5 2" stroke-width="1.8" />
  <path d="M4 3l2.5 2" stroke-width="1.5" />
  <path d="M20 3l-2.5 2" stroke-width="1.5" />""",

    "base64_workshop.svg": """  <rect x="3" y="4" width="7" height="16" rx="2" />
  <rect x="14" y="4" width="7" height="16" rx="2" />
  <path d="M5.5 9h2" />
  <path d="M5.5 12h2" />
  <path d="M5.5 15h2" />
  <path d="M16.5 9h2" />
  <path d="M16.5 12h2" />
  <path d="M16.5 15h2" />
  <path d="M10 12h4m-1.5-2l2 2-2 2" stroke-width="1.4" />""",

    "uuid_batch.svg": """  <rect x="3" y="6" width="18" height="12" rx="3" />
  <path d="M7 10v4" />
  <path d="M10 10v4" />
  <circle cx="14" cy="12" r="1.5" fill="currentColor" />
  <circle cx="17" cy="12" r="1.5" fill="currentColor" />
  <path d="M3 10h2" stroke-width="1.2" />
  <path d="M19 10h2" stroke-width="1.2" />""",

    "csv_cleaner.svg": """  <rect x="3" y="4" width="18" height="16" rx="2" />
  <path d="M3 10h18" stroke-width="1.4" />
  <path d="M3 15h18" stroke-width="1.4" />
  <path d="M9 4v16" stroke-width="1.4" />
  <path d="M15 4v16" stroke-width="1.4" />
  <path d="M17 6.5l2 2" stroke-width="1.5" />
  <circle cx="18" cy="7.5" r="1" fill="currentColor" />""",

    "jwt_inspector.svg": """  <path d="M12 2L4 5v6c0 5.5 3.8 10 8 11 4.2-1 8-5.5 8-11V5l-8-3z" />
  <path d="M8 9h8" stroke-width="1.3" />
  <path d="M8 12h8" stroke-width="1.3" />
  <path d="M9 15h6" stroke-width="1.3" />
  <circle cx="12" cy="12" r="1" fill="currentColor" />""",

    "regex_workbench.svg": """  <path d="M4 7l4 5-4 5" />
  <path d="M20 7l-4 5 4 5" />
  <path d="M12 8v8" stroke-width="1.5" />
  <path d="M9 10l6 4" stroke-width="1.5" />
  <path d="M15 10l-6 4" stroke-width="1.5" />"""
}

COMMON_ICONS = {
    "estop.svg": """  <polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2" stroke-width="2" />
  <line x1="12" y1="7" x2="12" y2="13" stroke-width="2.5" />
  <circle cx="12" cy="17" r="1.2" fill="currentColor" />""",

    "settings.svg": """  <circle cx="12" cy="12" r="3" />
  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />""",

    "search.svg": """  <circle cx="11" cy="11" r="7" stroke-width="1.8" />
  <path d="M21 21l-4.35-4.35" stroke-width="2" />""",

    "refresh.svg": """  <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
  <path d="M3 21v-5h5" />
  <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
  <path d="M21 3v5h-5" />""",

    "export.svg": """  <path d="M4 14v4a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-4" />
  <path d="M12 3v11" stroke-width="2" />
  <path d="M7 8l5-5 5 5" stroke-width="2" />""",

    "clear.svg": """  <path d="M3 6h18" stroke-width="1.6" />
  <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
  <line x1="10" y1="11" x2="10" y2="17" stroke-width="1.4" />
  <line x1="14" y1="11" x2="14" y2="17" stroke-width="1.4" />"""
}

def main():
    categories = [
        ("robot", ROBOT_ICONS),
        ("tools", TOOL_ICONS),
        ("common", COMMON_ICONS),
    ]

    total = 0
    for subdir, icons in categories:
        target_dir = BASE_DIR / subdir
        target_dir.mkdir(parents=True, exist_ok=True)
        for name, body in icons.items():
            filepath = target_dir / name
            content = SVG_HEADER + body + "\n" + SVG_FOOTER
            filepath.write_text(content, encoding="utf-8")
            total += 1
            print(f"Created: {filepath.relative_to(BASE_DIR.parent.parent)}")

    print(f"\\nSuccessfully generated {total} unified SVG icons!")

if __name__ == "__main__":
    main()
