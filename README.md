# Chronos Log

A graphical application for tracking work activities during your workday. Track time spent on different activities across projects and get daily summaries for entering into your time management system.

## Features

- **Time Tracking**: Log time entries with project/activity, duration (HH:MM format), and comments
- **Daily Summaries**: View aggregated time per activity for easy entry into time management systems
- **Project Management**: Add, edit, activate/deactivate, and delete projects
- **Activity Management**: Manage activities linked to projects
- **Date Navigation**: Easily switch between days to view/edit entries
- **Copy-to-Clipboard**: Quick copy of time totals for easy pasting
- **Persistent Storage**: SQLite database stores all data locally

## Building

### Prerequisites

- Rust toolchain (install from https://rustup.rs)
- For Linux: Development libraries for your system's graphics backend

**On CachyOS/Arch Linux:**
```bash
sudo pacman -S base-devel gtk3 glib2 cairo pango gdk-pixbuf2 atk
```

**On Windows 11:**
No additional dependencies needed - just install Rust.

### Build Commands

**Debug build (for development):**
```bash
cargo build
```

**Release build (optimized, for daily use):**
```bash
cargo build --release
```

### Running

**Development:**
```bash
cargo run
```

**Release:**
```bash
cargo run --release
# Or directly run the binary:
# Linux: ./target/release/chronos-log
# Windows: .\target\release\chronos-log.exe
```

## Cross-Compiling for Windows

To build on Linux for Windows 11:

1. Install the Windows target:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

2. Install MinGW-w64:
   ```bash
   sudo pacman -S mingw-w64-gcc
   ```

3. Build for Windows:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

The Windows executable will be at `target/x86_64-pc-windows-gnu/release/chronos-log.exe`

## Deploying to Windows 11

After cross-compiling from CachyOS/Linux, you can deploy the application to Windows 11:

1. **Copy the executable** from `target/x86_64-pc-windows-gnu/release/chronos-log.exe` to your Windows machine

2. **No additional dependencies needed** - The executable is self-contained with:
   - Bundled SQLite (no separate installation required)
   - All GUI libraries statically linked
   - MinGW runtime included

3. **First run on Windows**:
   - Double-click `chronos-log.exe` to launch
   - The application will automatically create its data directory at:
     ```
     %LOCALAPPDATA%\chronos-log\
     ```
   - Database file will be at: `%LOCALAPPDATA%\chronos-log\chronos_log.db`

4. **Optional - Create a shortcut**:
   - Right-click `chronos-log.exe` → "Create shortcut"
   - Move the shortcut to your Desktop or Start Menu folder
   - Optionally add a custom icon

5. **Optional - Add to Windows startup** (to launch on login):
   - Press `Win+R`, type `shell:startup`, press Enter
   - Copy a shortcut to `chronos-log.exe` into this folder

### Windows-Specific Notes

- **No console window** - In release builds, the application runs without showing a console window (configured with `windows_subsystem = "windows"` in `main.rs`)
- **Data persistence** - All data is stored locally in the Windows user's AppData folder
- **No internet required** - The application works completely offline
- **Antivirus** - If Windows Defender or other antivirus software flags the executable, you may need to add an exception (common with MinGW-compiled executables)

## Usage

### Time Tracking Tab

1. Select a project/activity from the dropdown
2. Enter time in HH:MM format (use +15m, +30m buttons for quick adjustments)
3. Add a comment describing what you did
4. Click "Add Entry"

### Daily Summary Tab

View the total time spent on each activity for the selected day. Use the "Copy" button next to each activity to copy the time total to your clipboard for pasting into your time management system.

### Projects Tab

- Create new projects with name and description
- Edit existing projects
- Activate/deactivate projects (deactivated projects won't appear in dropdowns)
- Delete projects (warning: this deletes all associated activities and time entries!)

### Activities Tab

- Create activities linked to specific projects
- Filter by project
- Activate/deactivate activities
- Delete activities

## Data Storage

The database is stored at:
- **Windows**: `%LOCALAPPDATA%\chronos-log\chronos_log.db`
- **Linux**: `~/.local/share/chronos-log/chronos_log.db`
- **macOS**: `~/Library/Application Support/chronos-log/chronos_log.db`

If the data directory cannot be created, the database will be stored in the current working directory.

## Example Project Structure

Based on your example:

**Project:** 33 - IT-Support
**Activities:**
- IT-Support - Trollhättan
- IT-Support - Göteborg
- IT-Support - Västerås
- IT-Support - Östersund

Each activity can have multiple time entries per day with individual comments. The Daily Summary view shows the total time per activity, which you can enter into your time management system.

## License

MIT License - feel free to modify and use as needed.
