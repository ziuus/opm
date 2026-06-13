# Orphan Process Manager (OPM)

A high-performance, lightweight terminal utility built in Rust to detect, monitor, and manage orphaned background processes that drain system resources.

## 📋 Overview

Modern development environments often leave behind "zombie" or orphaned processes—Node servers, headless browsers, and build tools—that continue to consume CPU and RAM long after their parent application (IDE or Terminal) has closed. 

OPM provides a real-time dashboard to identify these leaks and reclaim your system resources with a single keystroke.

## ✨ Features

- **Intelligent Detection**: Specifically targets orphaned processes reparented to PID 1 that match common development patterns (Node.js, Python, Chrome, etc.).
- **Network Port Mapping**: Automatically identifies which processes are holding local TCP ports, helping resolve "Address already in use" errors.
- **Resource Monitoring**: Real-time tracking of memory usage for every detected process.
- **Interactive Reaper**: A clean TUI (Terminal User Interface) to safely terminate processes individually.
- **Zero-Bloat**: Built with Rust and `ratatui` for minimal footprint and maximum performance.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Linux (optimized for `/proc` based process inspection)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/opm.git
   cd opm
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   ./target/release/opm
   ```

## 🎮 Controls

| Key | Action |
|-----|--------|
| `k` / `↑` | Move selection up |
| `j` / `↓` | Move selection down |
| `x` / `Enter` | Terminate selected process |
| `r` | Manual refresh |
| `q` / `Esc` | Exit application |

## 🛠️ Technology Stack

- **Core**: Rust
- **TUI Framework**: [Ratatui](https://github.com/ratatui/ratatui)
- **Process Inspection**: `sysinfo` and `procfs`
- **Terminal Backend**: `crossterm`

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
