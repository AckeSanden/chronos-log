# Chronos Log - Quick Deployment Reference

## Build on CachyOS

```bash
# Easy way (using provided script)
./build-windows.sh

# Manual way
cargo build --release --target x86_64-pc-windows-gnu
```

**Output**: `target/x86_64-pc-windows-gnu/release/chronos-log.exe` (~6 MB)

---

## Deploy to Windows 11

### Method 1: Simple Copy
1. Copy `chronos-log.exe` to Windows machine
2. Place anywhere you like (Desktop, Program Files, etc.)
3. Double-click to run - done!

### Method 2: Deployment Package
```bash
# Run the build script and answer 'y' when prompted
./build-windows.sh
# Creates: chronos-log-windows-YYYYMMDD.zip
```

Transfer the zip file to Windows, extract, and run.

---

## First Run on Windows

- ✅ No installation required
- ✅ No admin rights needed
- ✅ No additional downloads
- ✅ Example data auto-created
- 📁 Data stored in: `%LOCALAPPDATA%\chronos-log\`

---

## Quick Tests Before Deployment

### On CachyOS (after building)
```bash
# Verify executable exists
ls -lh target/x86_64-pc-windows-gnu/release/chronos-log.exe

# Check it's a Windows PE executable
file target/x86_64-pc-windows-gnu/release/chronos-log.exe
# Should show: "PE32+ executable for MS Windows"

# Check size (should be ~6 MB)
du -h target/x86_64-pc-windows-gnu/release/chronos-log.exe
```

### On Windows 11 (after transfer)
1. Double-click `chronos-log.exe`
2. Window should open immediately
3. See example projects: "33 - IT-Support" and "40 - Development"
4. Try adding a time entry
5. Check data was saved by reopening the app

---

## Common Deployment Scenarios

### Scenario 1: Personal Use
- Copy .exe to `C:\Program Files\ChronosLog\` or `C:\Apps\chronos-log\`
- Create Desktop shortcut
- Done!

### Scenario 2: Portable/USB Drive
- Copy .exe to USB drive folder
- Database will be created next to .exe
- Run from any Windows machine

### Scenario 3: Multi-User/Work Computer
- Each user gets their own database in their `%LOCALAPPDATA%`
- Copy .exe to shared location: `C:\Program Files\ChronosLog\`
- All users can access the same executable

### Scenario 4: Auto-Start with Windows
1. Copy .exe to desired location
2. Press `Win+R`, type `shell:startup`
3. Create shortcut to .exe in startup folder

---

## Troubleshooting

### Windows Defender blocks it
**Fix**: Add exception in Windows Security → Virus & threat protection

### Won't start / Shows error
**Check**: 
- Right-click .exe → Properties → Unblock (if present)
- Run from Command Prompt to see errors

### Database not found after moving .exe
**Fix**: Database is in `%LOCALAPPDATA%\chronos-log\` - it stays there even if you move the .exe

---

## Update Workflow

```bash
# On CachyOS: Make changes, rebuild
cargo build --release --target x86_64-pc-windows-gnu

# On Windows: Close app, replace .exe, restart
# Database is preserved automatically
```

---

## File Sizes Reference

| File | Size |
|------|------|
| chronos-log.exe | ~6 MB |
| chronos_log.db (empty) | ~20 KB |
| chronos_log.db (1 year data) | ~100-500 KB |

---

## Transfer Methods

| Method | Speed | Notes |
|--------|-------|-------|
| USB Drive | Fast | Most reliable |
| Network Share | Fast | Good for same network |
| Email | Slow | May be blocked by antivirus |
| Cloud Storage | Medium | Dropbox, Google Drive, OneDrive |
| Git/GitHub | Medium | Version control bonus |

---

## Pre-Deployment Checklist

- [ ] Code compiles without errors: `cargo check`
- [ ] Windows build succeeds: `cargo build --release --target x86_64-pc-windows-gnu`
- [ ] Executable exists and is ~6 MB
- [ ] File command shows "PE32+ executable for MS Windows"
- [ ] No debug features enabled (check Cargo.toml profile.release)
- [ ] README and documentation updated

---

## Post-Deployment Verification

On Windows machine:

- [ ] .exe launches without errors
- [ ] Window opens and UI renders correctly
- [ ] Can create a project
- [ ] Can create an activity
- [ ] Can add a time entry
- [ ] Data persists after closing and reopening
- [ ] Date selector works
- [ ] Daily summary shows entries
- [ ] Edit and delete functions work

---

## Backup Strategy

### For Development
```bash
# Backup entire project
tar -czf chronos-log-backup-$(date +%Y%m%d).tar.gz chronos-log/
```

### For Windows Users
**Location**: `%LOCALAPPDATA%\chronos-log\chronos_log.db`

**Schedule**: 
- Weekly: Copy to different folder
- Monthly: Copy to external drive
- Before updates: Always backup

---

## Quick Command Reference

```bash
# Build for Linux (development)
cargo run

# Build for Windows (deployment)
cargo build --release --target x86_64-pc-windows-gnu

# Check for errors
cargo check

# Run tests
cargo test

# Clean build artifacts
cargo clean

# Update dependencies
cargo update
```

---

## Performance Expectations

| Metric | Value |
|--------|-------|
| Startup time | < 1 second |
| Memory usage | 50-80 MB |
| CPU (idle) | < 1% |
| CPU (active) | 1-5% |
| Disk I/O | Minimal |

---

## Need More Help?

- **Detailed Windows guide**: See `WINDOWS.md`
- **General usage**: See `README.md`
- **Code documentation**: See inline comments in `src/`

---

**Last Updated**: 2025-01-11
**Build Target**: x86_64-pc-windows-gnu
**Minimum Windows**: Windows 10 (64-bit) / Windows 11