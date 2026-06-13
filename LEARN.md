# Learning OPM: A Step-by-Step Guide

Welcome to the OPM tutorial. This guide will help you understand how to use and extend the Orphan Process Manager.

## 🛠️ Step 1: Installation & Setup

Ensure you have the Rust toolchain installed. 

```bash
git clone https://github.com/ziuus/opm.git
cd opm
cargo build
```

## 🔍 Step 2: Understanding "Orphans"

In Linux, when a parent process dies, its children are inherited by PID 1 (`systemd` or `init`). OPM scans the system for these specific processes.

Run OPM to see what's currently "orphaned" on your system:
```bash
cargo run
```

## 🖥️ Step 3: Navigating the TUI

- Use `j` and `k` to scroll through the list.
- Look at the **Memory MB** column to see which processes are the heaviest.
- Look at the **Ports** column to see which process is blocking your `localhost:3000`.

## 💀 Step 4: Reclaiming Resources

When you find a process that shouldn't be running (e.g., a `node` server from a project you closed an hour ago):
1.  Select it with the arrow keys.
2.  Press `Enter` or `x`.
3.  The process is terminated, and the list refreshes.

## 🚀 Next Steps: Customizing Detection

You can add more process names to the detection list in `src/main.rs`. Look for the `suspicious_names` vector and add your own tools!
