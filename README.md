# SectorForge

> Cross-platform disk cloning and imaging tool built with Tauri 2 + Vue 3.

SectorForge is a fast, native disk cloner for Windows and Linux. It performs sector-by-sector copies with SHA-256 verification, progress tracking, and a clean dark-themed UI.

## Features

- **Clone to Image** – Create raw disk images (`.img`, `.raw`, `.dd`) from any physical disk
- **Restore from Image** – Write a disk image back to a physical disk
- **Disk-to-Disk Copy** – Clone one physical disk directly to another
- **SHA-256 Verify** – Compute and verify hashes for any disk image
- **Progress Tracking** – Real-time speed, ETA, and percentage during operations
- **Cross-Platform** – Windows and Linux support with native disk enumeration (no PowerShell)
- **Admin Detection** – Warns and provides one-click restart if elevation is needed
- **Cancel Anytime** – Gracefully stop an ongoing operation

## Screenshots

*(Screenshots will be added in a future release)*

## Requirements

### Windows
- Windows 10/11 (64-bit)
- Administrator privileges required for physical disk access

### Linux
- Kernel 5.x+ recommended
- Root privileges (`sudo`) required for physical disk access

## Installation

### Download Pre-built Binaries

Visit the [Releases](https://github.com/mpesteban/sector-forge/releases) page and download the latest version for your platform.

**Windows options:**
- `sector-forge_<version>_x64-setup.exe` – Installer with auto-updater support
- `sector-forge_<version>_x64_en-US.msi` – MSI installer for enterprise deployment
- `sector-forge.exe` – Portable single binary (no installation required)

### Build from Source

#### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (LTS) or [Bun](https://bun.sh/)

#### Clone & Build

```bash
# Clone the repository
git clone https://github.com/mpesteban/sector-forge.git
cd sector-forge

# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build release binaries
bun run tauri build
```

Release artifacts will be generated in `src-tauri/target/release/bundle/`.

## Usage

1. **Launch SectorForge** with Administrator (Windows) or `sudo` (Linux) privileges
2. **Select a disk** using the dialog picker
3. **Choose your operation** tab: Clone, Restore, Disk-to-Disk, or Verify
4. **Set paths** and options (e.g., enable SHA-256 verification)
5. **Click Start** and monitor progress in real time
6. **Cancel anytime** if needed

> **Warning:** Restoring or disk-to-disk operations will **permanently destroy** all data on the target disk. Triple-check your selection before starting.

## Architecture

```
Frontend  (Vue 3 + Vite)
  ├── src/components/        UI panels, disk dialog, progress display
  ├── src/composables/       Typed API layer, progress listeners
  ├── src/types/             Shared TypeScript definitions
  └── src/styles/            Global CSS variables & utilities

Backend   (Rust + Tauri 2)
  ├── src-tauri/src/lib.rs        Command registration, app state
  ├── src-tauri/src/disk.rs       Cross-platform disk enumeration
  ├── src-tauri/src/clone.rs      Sector-by-sector clone/restore/verify
  └── src-tauri/src/state.rs      Atomic cancellation & run flags
```

## Platform Notes

### Windows
- Raw disk paths use `\\.\PhysicalDriveN` format internally
- Drive letters (C:, D:) are displayed for user-friendly selection
- If the app is not elevated, a banner appears with a **Restart as Administrator** button

### Linux
- Disks are enumerated from `/sys/block/` and `/proc/mounts`
- Virtual devices (loop, ram, zram, dm-, sr) are filtered out automatically

## Development

```bash
# Start frontend dev server
bun run dev

# Start Tauri dev window with hot reload
bun run tauri dev

# Run TypeScript checks
bun run build

# Run Rust checks
cd src-tauri && cargo check
```

## Tech Stack

| Layer       | Technology                                    |
|-------------|-----------------------------------------------|
| Framework   | [Tauri 2](https://tauri.app/)                 |
| Frontend    | [Vue 3](https://vuejs.org/) + Composition API |
| Build Tool  | [Vite](https://vitejs.dev/)                   |
| Styling     | CSS Variables (no framework)                  |
| Validation  | [Vee-Validate](https://vee-validate.logaretm.com/) + [Zod](https://zod.dev/) |
| Hashing     | SHA-256 via `sha2` crate                      |

## Contributing

Contributions are welcome! Please open an issue first to discuss changes before submitting a PR.

## License

SectorForge is released under the [MIT License](LICENSE).