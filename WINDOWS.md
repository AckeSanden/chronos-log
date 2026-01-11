# Windows 11 Deployment Guide

This guide covers deploying and running Chronos Log on Windows 11 after building it on CachyOS/Linux.

## Quick Start

1. **Build the Windows executable** (on CachyOS):
   ```bash
   ./build-windows.sh
   ```
   Or manually:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnu
   ```

2. **Copy to Windows**: Transfer `target/x86_64-pc-windows-gnu/release/chronos-log.exe` to your Windows 11 machine

3. **Run**: Double-click `chronos-log.exe` - no installation needed!

## What's Included

The Windows executable is **completely self-contained**:

- ✅ No external dependencies required
- ✅ SQLite is bundled (no separate installation)
- ✅ All GUI libraries are statically linked
- ✅ MinGW runtime included
- ✅ Works offline (no internet connection needed)
- ✅ No admin rights required

## Data Storage on Windows

### Default Location
```
%LOCALAPPDATA%\chronos-log\chronos_log.db
```

Typically expands to:
```
C:\Users\YourUsername\AppData\Local\chronos-log\chronos_log.db
```

### Fallback Location
If the AppData directory cannot be created, the database will be stored in the same folder as the executable:
```
chronos_log.db (next to chronos-log.exe)
```

## First Run

When you first launch Chronos Log on Windows:

1. The application window opens immediately
2. A data directory is automatically created in `%LOCALAPPDATA%\chronos-log\`
3. A new SQLite database is initialized
4. Example projects and activities are created for you to explore
5. You can immediately start tracking time or customize the examples

### Example Data Created
- **Project**: "33 - IT-Support" with activities for different locations
- **Project**: "40 - Development" with development-related activities

You can edit or delete these and create your own.

## Windows-Specific Features

### No Console Window
In release builds, the application runs as a pure GUI application with no console window showing in the background. This is configured in the code with:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

### System Integration Options

#### Add to Startup (Auto-launch on login)
1. Press `Win+R`
2. Type: `shell:startup` and press Enter
3. Create a shortcut to `chronos-log.exe` in this folder
4. The app will now start automatically when you log in

#### Add to Start Menu
1. Right-click on `chronos-log.exe`
2. Select "Pin to Start"

Or manually:
1. Create a shortcut to `chronos-log.exe`
2. Move it to: `%APPDATA%\Microsoft\Windows\Start Menu\Programs\`

#### Add to Taskbar
- Right-click on `chronos-log.exe` → "Pin to taskbar"

## Troubleshooting

### Windows Defender / Antivirus Warning

**Issue**: Windows Defender or your antivirus software flags the executable as suspicious.

**Why**: MinGW-compiled executables sometimes trigger false positives because they're less common than MSVC-compiled programs.

**Solutions**:
1. **Add an exception** in Windows Defender:
   - Open Windows Security → Virus & threat protection
   - Click "Manage settings" → "Add or remove exclusions"
   - Add `chronos-log.exe` or the folder containing it

2. **Alternative**: Build with MSVC target instead (requires more setup)
   ```bash
   cargo build --release --target x86_64-pc-windows-msvc
   ```

### Application Won't Start

**Issue**: Double-clicking does nothing or shows an error.

**Check**:
1. Verify you have 64-bit Windows 11 (the executable is 64-bit)
2. Try running from Command Prompt to see error messages:
   ```cmd
   cd path\to\chronos-log.exe
   chronos-log.exe
   ```
3. Ensure Windows is up to date (some older Windows versions may be missing system libraries)

### Can't Find Database / Data Missing

**Check these locations**:
1. `%LOCALAPPDATA%\chronos-log\chronos_log.db`
2. Same folder as `chronos-log.exe\chronos_log.db` (fallback)

**View the path**: Press `Win+R`, type `%LOCALAPPDATA%\chronos-log`, press Enter

### GUI Looks Blurry / Scaling Issues

**Issue**: Text or UI elements appear blurry on high-DPI displays.

**Solution**:
1. Right-click `chronos-log.exe` → Properties
2. Go to "Compatibility" tab
3. Click "Change high DPI settings"
4. Check "Override high DPI scaling behavior"
5. Select "System" or "System (Enhanced)"

## Backup Your Data

### Manual Backup
Copy the database file:
```
%LOCALAPPDATA%\chronos-log\chronos_log.db
```
to a backup location (USB drive, cloud storage, etc.)

### Restore from Backup
Simply replace the database file with your backup copy while the application is closed.

## Updating the Application

1. Close the running application
2. Replace `chronos-log.exe` with the new version
3. Your data (database) is preserved automatically
4. Restart the application

**Note**: Always backup your database before updating!

## Performance Notes

- **Startup time**: < 1 second on modern systems
- **Memory usage**: ~50-80 MB typical
- **CPU usage**: Minimal (< 1% when idle)
- **Disk usage**: 
  - Executable: ~6 MB
  - Database: < 1 MB (grows with data)

## Building Notes (For Developers)

If you need to rebuild or modify:

### Prerequisites on CachyOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Install MinGW cross-compiler
sudo pacman -S mingw-w64-gcc
```

### Build Commands
```bash
# Quick build (using script)
./build-windows.sh

# Manual build
cargo build --release --target x86_64-pc-windows-gnu

# Output location
target/x86_64-pc-windows-gnu/release/chronos-log.exe
```

### Alternative: Build with MSVC (more complex)
Requires setting up cross-compilation with `cargo-xwin`:
```bash
cargo install cargo-xwin
cargo xwin build --release --target x86_64-pc-windows-msvc
```

MSVC builds may have better Windows integration but require more setup.

## Security Considerations

- **No network access**: The application doesn't connect to the internet
- **No telemetry**: No data is sent anywhere
- **Local storage only**: All data stays on your machine
- **No admin rights**: Runs in user space only
- **Open source**: Code can be audited (see repository)

## Portable Installation

To run Chronos Log as a portable application:

1. Copy `chronos-log.exe` to a folder (e.g., on a USB drive)
2. The database will be created in the same folder if AppData isn't available
3. You can move the folder freely - just keep the .exe and .db together

**Tip**: Create a `portable.txt` file next to the executable if you want to force portable mode (requires code modification to check for this file).

## Uninstalling

To completely remove Chronos Log:

1. Delete `chronos-log.exe`
2. Delete the data directory: `%LOCALAPPDATA%\chronos-log\`
3. Remove any shortcuts you created
4. Remove from startup folder if applicable

That's it - no registry entries or hidden files to worry about!

## Support

If you encounter issues:

1. Check this document first
2. Check the main README.md for general usage
3. Try rebuilding with the latest code
4. Check Windows Event Viewer for crash details

## License

MIT License - See LICENSE file for details.