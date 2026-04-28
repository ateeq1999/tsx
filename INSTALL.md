# tsx — Installation Guide

Complete step-by-step guide to build, install, and run tsx (CLI + Registry Server) on Linux, macOS, and Windows.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Install tsx CLI](#install-tsx-cli)
3. [Build from Source](#build-from-source)
4. [Install Registry Server](#install-registry-server)
5. [Docker Setup](#docker-setup)
6. [Development Setup](#development-setup)
7. [Make Binaries Globally Available](#make-binaries-globally-available)
8. [Verify Installation](#verify-installation)
9. [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Tool | Minimum Version | Install |
|------|----------------|---------|
| Rust (with Cargo) | 1.88+ | `rustup update stable` |
| PostgreSQL | 14+ | [postgresql.org](https://www.postgresql.org/download/) or use Neon free tier |
| Docker (optional) | 20+ | [docker.com](https://docs.docker.com/get-docker/) |
| Git | 2.30+ | `apt install git` / `brew install git` |

Check your versions:
```bash
rustc --version   # Should be 1.88 or higher
cargo --version
psql --version    # For registry server only
docker --version  # For Docker setup only
```

---

## Install tsx CLI

### Method 1: Cargo (All Platforms)

```bash
cargo install tsx --locked
```

### Method 2: Pre-built Binary (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/ateeq1999/tsx/main/scripts/install.sh | sh
```

Supported platforms:
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- Linux ARM64 (`aarch64-unknown-linux-gnu`)
- macOS x86_64 (`x86_64-apple-darwin`)
- macOS ARM64 (`aarch64-apple-darwin`)

### Method 3: Winget (Windows)

```powershell
winget install tsx
```

### Method 4: Build from Source (All Platforms)

See [Build from Source](#build-from-source) section below.

---

## Build from Source

### Step 1: Clone the Repository

```bash
git clone https://github.com/ateeq1999/tsx.git
cd tsx
```

### Step 2: Build Everything (Workspace)

```bash
# Build all crates (CLI, registry server, and tools)
cargo build --release

# Binaries output:
#   - CLI:              target/release/tsx (Linux/macOS) or target/release/tsx.exe (Windows)
#   - Registry Server:   target/release/tsx-registry (Linux/macOS) or target/release/tsx-registry.exe (Windows)
#   - Other tools:       target/release/tsx-forge, target/release/tsx-watcher, etc.
```

### Step 3: Build Specific Components

```bash
# Build only the CLI
cargo build --release -p tsx

# Build only the registry server
cargo build --release -p tsx-registry

# Build only the codegen engine
cargo build --release -p tsx-forge
```

### Platform-Specific Notes

**Linux:**
```bash
# If you get OpenSSL errors:
sudo apt install pkg-config libssl-dev   # Debian/Ubuntu
sudo dnf install pkg-config openssl-devel # Fedora
```

**macOS:**
```bash
# Install OpenSSL if needed (using Homebrew):
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
cargo build --release
```

**Windows:**
```powershell
# Use PowerShell or Git Bash
# If you get OpenSSL errors, set environment variables:
# Download OpenSSL for Windows or use vcpkg
```

---

## Install Registry Server

The registry server (`tsx-registry`) is the backend for hosting tsx packages.

### Method 1: Cargo Install

```bash
cargo install --path crates/registry-server --locked
```

### Method 2: Docker (Recommended for Production)

See [Docker Setup](#docker-setup) below.

### Method 3: Build from Source

```bash
cd tsx
cargo build --release -p tsx-registry
# Binary: target/release/tsx-registry
```

### Registry Server Configuration

Copy the example environment file and configure it:

```bash
cp crates/registry-server/.env.example crates/registry-server/.env
# Edit .env and set your DATABASE_URL
```

Required environment variables:
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/tsx_registry
PORT=8282                              # Optional, default: 8282
TSX_REGISTRY_API_KEY=your-secret-key   # Optional, for admin endpoints
DATA_DIR=/data                          # Optional, for tarball storage
RUST_LOG=tsx_registry=info            # Optional, log level
```

---

## Docker Setup

### Quick Start (Registry Server + PostgreSQL)

```bash
cd crates/registry-server
docker compose up -d
```

Services:
- **Registry Server**: `http://localhost:8282`
- **PostgreSQL**: `localhost:5433` (user: `tsx`, password: `tsx`, database: `tsx_registry`)

### Build Docker Image Manually

```bash
# Build the image
docker build -f crates/registry-server/Dockerfile -t tsx-registry .

# Run with custom environment
docker run -d \
  -p 8282:8282 \
  -e DATABASE_URL="postgresql://tsx:tsx@host.docker.internal:5433/tsx_registry" \
  -e PORT=8282 \
  -v $(pwd)/data:/data \
  --name tsx-registry \
  tsx-registry
```

### Docker Compose Configuration

The `docker-compose.yml` file is located at `crates/registry-server/docker-compose.yml`. You can customize:
- Ports (default: registry `8282`, PostgreSQL `5433`)
- Volume mounts for data persistence
- Environment variables

---

## Development Setup

### Step 1: Clone and Build

```bash
git clone https://github.com/ateeq1999/tsx.git
cd tsx
cargo build
```

### Step 2: Configure Registry Server (Optional)

```bash
cp crates/registry-server/.env.example crates/registry-server/.env
# Edit .env to add your DATABASE_URL
```

### Step 3: Run Tests

```bash
# All tests
cargo test

# CLI tests only
cargo test -p tsx

# Registry server tests only
cargo test -p tsx-registry

# E2E tests
cargo test -p tsx --test e2e
```

### Step 4: Lint and Format

```bash
# Check code style
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Auto-fix formatting
cargo fmt
```

### Step 5: Run Locally

```bash
# Run CLI directly (development mode)
cargo run -p tsx -- --version

# Run registry server (development mode)
cargo run -p tsx-registry

# With custom env file:
cargo run -p tsx-registry -- --config crates/registry-server/.env
```

---

## Make Binaries Globally Available

### Linux/macOS

**Method 1: Copy to `/usr/local/bin` (System-wide)**
```bash
sudo cp target/release/tsx /usr/local/bin/tsx
sudo cp target/release/tsx-registry /usr/local/bin/tsx-registry
```

**Method 2: Symlink to `~/.local/bin` (User-only)**
```bash
mkdir -p ~/.local/bin
ln -sf $(pwd)/target/release/tsx ~/.local/bin/tsx
ln -sf $(pwd)/target/release/tsx-registry ~/.local/bin/tsx-registry
```

**Method 3: Add Cargo bin to PATH**
```bash
# Add to ~/.bashrc or ~/.zshrc:
export PATH="$HOME/.cargo/bin:$PATH"

# Then you can run:
tsx --version
tsx-registry
```

### Windows

**Method 1: PowerShell (Admin)**
```powershell
# Copy binaries to a system directory
Copy-Item target\release\tsx.exe C:\Windows\System32\
Copy-Item target\release\tsx-registry.exe C:\Windows\System32\
```

**Method 2: Add Cargo bin to System PATH (setx)**

Using `setx` command (from Microsoft docs: https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/setx):

```powershell
# View current system PATH
setx /M PATH

# Add Cargo bin to system PATH (run as Admin)
setx /M PATH "%PATH%;%USERPROFILE%\.cargo\bin"

# Or using PowerShell:
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$newPath = $currentPath + ";$env:USERPROFILE\.cargo\bin"
setx /M PATH "$newPath"
```

> **Note:** After using `setx`, restart PowerShell/Terminal for changes to take effect.

**Method 3: Add Project Binaries to User PATH (setx)**

```powershell
# Add project binaries to user PATH
setx PATH "%PATH%;D:\DEV\open-source\tsx\target\release"

# Or using PowerShell:
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
$newPath = $currentPath + ";D:\DEV\open-source\tsx\target\release"
[Environment]::SetEnvironmentVariable("Path", "$newPath", "User")
```

**Method 4: Copy Binaries to System Directory**

```powershell
# Copy binaries to a directory already in PATH (run as Admin)
Copy-Item target\release\tsx.exe C:\Windows\System32\
Copy-Item target\release\tsx-registry.exe C:\Windows\System32\
```

**Method 3: Add Project Binaries to User PATH**
```powershell
$projectBin = "D:\DEV\open-source\tsx\target\release"
[Environment]::SetEnvironmentVariable("Path", "$env:Path;$projectBin", "User")
```

---

## Verify Installation

### Verify tsx CLI

```bash
tsx --version
# Should output: tsx x.x.x

tsx --help
# Should show available commands
```

### Verify Registry Server

```bash
# If installed via cargo or built from source:
tsx-registry --help
# Should show help message

# If running via Docker:
curl http://localhost:8282/health
# Should return: {"status":"ok"}
```

### Quick Test

```bash
# Test CLI code generation
tsx run add-schema --json '{"name":"test","fields":[{"name":"title","type":"string"}]}'

# Test registry API (if running)
curl http://localhost:8282/v1/packages
```

---

## Troubleshooting

### Error: `linker cc not found` (Linux)

```bash
sudo apt install build-essential   # Debian/Ubuntu
sudo dnf groupinstall "Development Tools"   # Fedora
```

### Error: `pkg-config not found` (Linux)

```bash
sudo apt install pkg-config libssl-dev   # Debian/Ubuntu
sudo dnf install pkg-config openssl-devel # Fedora
```

### Error: `generation expression is not immutable` (Registry Server)

This is a PostgreSQL migration issue. Reset the database:
```bash
cd crates/registry-server
docker compose down -v
docker compose up -d
```

### Error: `port is already allocated`

Change the port in `crates/registry-server/docker-compose.yml`:
```yaml
ports:
  - "5434:5432"  # Change host port
```

### Windows: `cargo build` fails with OpenSSL errors

Use vcpkg to install OpenSSL:
```powershell
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg install openssl:x64-windows
set OPENSSL_DIR=C:\path\to\vcpkg\installed\x64-windows
```

### Docker: `no space left on device`

Clean up Docker:
```bash
docker system prune -a
docker volume prune
```

---

## Project Structure

```
tsx/
├── crates/
│   ├── cli/              # tsx CLI binary (25+ commands)
│   ├── registry-server/  # tsx-registry HTTP server (16 REST endpoints)
│   ├── forge/           # Template rendering engine (Tera-based)
│   ├── codegen/         # Code generation library
│   ├── shared/          # Shared types (CLI + server)
│   ├── lsp/             # Language Server Protocol (future)
│   ├── tui/             # Terminal UI (future)
│   ├── fmt/             # Formatter (future)
│   └── watcher/         # File watcher
├── packages/            # tsx packages (npm)
├── patterns/            # Pattern definitions
├── migrations/          # Database migrations
├── scripts/             # Install scripts
└── target/              # Build output (gitignored)
```

---

## Additional Resources

- [README.md](README.md) — Project overview and usage
- [CONTRIBUTING.md](CONTRIBUTING.md) — Contribution guidelines
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [Registry Server Env Example](crates/registry-server/.env.example) — Configuration reference

---

**Need help?** Open an issue at [github.com/ateeq1999/tsx/issues](https://github.com/ateeq1999/tsx/issues)
