import pathlib
p = pathlib.Path('src/views/ui_kit.rs')
lines = p.read_text(encoding='utf-8').split('\n')

# Find and remove duplicate #[allow(dead_code)] before AppTheme
for i in range(min(10, len(lines))):
    if lines[i].strip() == '#[allow(dead_code)]' and i + 1 < len(lines) and '#[derive' in lines[i+1]:
        lines.pop(i)
        print(f"Removed duplicate allow at line {i+1}")
        break

# Find styled_button and add #[allow(dead_code)]
for i, line in enumerate(lines):
    if 'pub fn styled_button(' in line:
        if i == 0 or '#[allow(dead_code)]' not in lines[i-1]:
            lines.insert(i, '#[allow(dead_code)]')
            print(f"Added allow before styled_button at line {i+1}")
        break

p.write_text('\n'.join(lines), encoding='utf-8')
print("Done")
