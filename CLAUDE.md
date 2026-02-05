# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

quicssh-rs is a QUIC proxy for SSH connections that enables SSH over QUIC protocol. It's a Rust implementation of quicssh that provides connection stability and migration capabilities for SSH sessions over unreliable networks.

## Commands

### Build and Development
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the project
cargo run -- server  # or client

# Check code
cargo check

# Format code
cargo fmt

# Run lints
cargo clippy

# Run tests
cargo test
```

### Usage
```bash
# Start server (listens on 0.0.0.0:4433 by default, proxies to 127.0.0.1:22)
cargo run -- server
cargo run -- server --listen 0.0.0.0:4433 --proxy-to 127.0.0.1:22

# Start server with custom MTU upper bound
cargo run -- server --mtu-upper-bound 1200
cargo run -- server --mtu-upper-bound safety  # Use RFC-compliant 1200 bytes

# Run client
cargo run -- client quic://hostname:4433

# Run client with custom MTU upper bound
cargo run -- client --mtu-upper-bound 1200 quic://hostname:4433
cargo run -- client --mtu-upper-bound safety quic://hostname:4433
```

## Architecture

The application consists of three main modules:

### Core Components
- **main.rs**: Entry point with CLI parsing using clap, handles subcommands and logging configuration
- **server.rs**: QUIC server that accepts connections and proxies to SSH server
- **client.rs**: QUIC client that connects to server and tunnels SSH traffic

### Network Flow
1. SSH client connects to quicssh-rs client via ProxyCommand
2. quicssh-rs client establishes QUIC connection to quicssh-rs server
3. quicssh-rs server proxies traffic to actual SSH server over TCP
4. QUIC provides connection migration and better weak network handling

### Key Libraries
- **quinn**: QUIC protocol implementation
- **tokio**: Async runtime
- **rustls**: TLS/QUIC crypto (with self-signed certs for QUIC)
- **clap**: CLI argument parsing
- **log4rs**: Logging framework

### Security Notes
- Server uses self-signed certificates generated via rcgen
- **Client skips certificate verification by default** (for ease of use with self-signed certs)
  - SSH layer provides end-to-end encryption and host key verification
  - QUIC acts as transport tunnel (similar to TCP)
  - Risk: DNS/IP spoofing could enable traffic interception (but not plaintext exposure due to SSH)
- Both modules handle MTUD (Maximum Transmission Unit Discovery) where supported

> Note: Certificate verification flags (`--verify-cert` on client, `--cert`/`--key` on server) are **not implemented yet**. Until they ship, deployment should assume QUIC cert verification is disabled.

<!-- TODO: Implement certificate verification option
Currently, the client always skips QUIC certificate verification (dangerous_configuration).
While SSH provides its own security layer, QUIC cert verification would prevent:
- DNS/IP spoofing attacks
- Traffic interception for future cryptanalysis
- Man-in-the-middle positioning

IMPORTANT: This requires BOTH server and client changes:

Server-side changes (src/server.rs):
1. Add --cert <path> option to specify TLS certificate file (PEM format)
2. Add --key <path> option to specify private key file (PEM format)
3. Modify configure_server() to:
   - Load cert/key from files when options are provided
   - Fall back to self-signed certificate (current behavior) when not specified
4. Support proper hostnames in self-signed cert (not just "localhost")

Client-side changes (src/client.rs):
1. Add --verify-cert flag to enable certificate verification (default: false)
2. Add --ca-cert <path> option to specify custom CA certificate
3. Modify make_client_endpoint() to:
   - Use rustls::RootCertStore with system certs when --verify-cert is set
   - Support custom CA cert for self-signed server certificates
   - Continue using SkipServerVerification when flag is absent (backward compatibility)
4. Update README to recommend --verify-cert for production use

Example usage:
  # Server with Let's Encrypt certificate
  quicssh-rs-robust server --cert /etc/letsencrypt/live/example.com/fullchain.pem \
                           --key /etc/letsencrypt/live/example.com/privkey.pem

  # Client with system CA verification
  quicssh-rs-robust client --verify-cert quic://example.com:4433

  # Self-signed certificate workflow
  quicssh-rs-robust server  # generates self-signed cert, prints fingerprint
  quicssh-rs-robust client --verify-cert --ca-cert /path/to/server.crt quic://hostname:4433

References:
  - src/server.rs:35-61 (configure_server, self-signed cert generation)
  - src/client.rs:147-161 (SkipServerVerification implementation)
-->

## Configuration

- Default server listen address: `0.0.0.0:4433`
- Default SSH proxy target: `127.0.0.1:22`
- Logging can be configured via `--log` and `--log-level` flags
- Test suite includes unit tests and integration tests (smoke test)

## Coding Guidelines

### Comments and Documentation

- **Always write comments in English**, not Japanese
- **Avoid emojis in code and metadata files** (Cargo.toml, comments, etc.)
  - Use plain text alternatives (e.g., "WARNING:" instead of "⚠️")
  - Emojis are acceptable in README.md for visual emphasis
- Reference relevant RFCs and standards when applicable
- Example: When setting MTU values, cite RFC 9000, RFC 8899, or RFC 8200

### Commit Messages

- **Always write commit messages in English**
- Follow conventional commits format: `type(scope): description`
- Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`
- Keep the first line under 72 characters
- Reference issue numbers when applicable

### MTU Configuration

The MTU (Maximum Transmission Unit) upper bound can be configured via the `--mtu-upper-bound` option:

**Default behavior (no option specified):**
- Uses Quinn's default MTU discovery with upper bound of **1452 bytes**
- Suitable for most standard network environments

**Conservative mode (`--mtu-upper-bound safety` or `--mtu-upper-bound 1200`):**
- Sets MTU upper bound to **1200 bytes** for maximum compatibility
- **RFC 9000 Section 14.1**: QUIC Initial packets must be at least 1200 bytes
- **RFC 8899 Section 5.1.2**: Recommends 1200 bytes as BASE_PLPMTU for UDP
- **RFC 8200**: IPv6 minimum link MTU is 1280 bytes
  - 1200-byte payload + 40-byte IPv6 header + 8-byte UDP header = 1248 bytes (fits within 1280)
- Ensures compatibility across all network environments, including VPN tunnels and IPv6-only networks

**Custom MTU:**
- Specify any numeric value (e.g., `--mtu-upper-bound 1300`)
- Useful for specific network requirements or testing

**Usage examples:**
```bash
# Server with safety MTU
quicssh-rs-robust server --mtu-upper-bound safety

# Client with custom MTU
quicssh-rs-robust client --mtu-upper-bound 1300 quic://hostname:4433

# Server with default Quinn MTU (no option)
quicssh-rs-robust server
```

## CI/CD Pipeline

The project uses GitHub Actions for automated testing, building, and releasing across multiple platforms.

### Workflows

The project has two GitHub Actions workflows with distinct purposes:

#### Lint Workflow ([lint.yml](.github/workflows/lint.yml))
Fast code style and quality checks on every push and PR:
- **Code formatting**: `cargo fmt --check` ensures consistent code style
- **Clippy lints**: `cargo clippy` provides suggestions (warnings allowed)
- **Purpose**: Quick feedback for developers during development

#### Release Workflow ([release.yml](.github/workflows/release.yml))
Comprehensive testing, building, and releasing. Consists of four main jobs:

<!-- TODO: Optimize workflow triggers to reduce unnecessary runs
Currently, the release workflow runs on every push to feature branches, which triggers:
- Full test suite (cargo test + clippy -D warnings)
- 9-platform cross-compilation build matrix (~10-15 minutes)

This may be excessive for routine development. Consider:
1. Limit release workflow to only: main branch, PRs, and release tags
2. Keep lint workflow for quick feedback on all branches
3. Developers can rely on lint workflow during feature development

Proposed trigger change for release.yml:
  on:
    push:
      branches: [main, master]
      tags: ['v*', 'test-release*']
    pull_request:

This would:
- Reduce CI time/cost for feature branch pushes
- Maintain full validation for PRs and releases
- Keep fast feedback via lint workflow on all branches
-->

**1. Test Job**
Runs on every push and pull request:
- **Platform**: Linux (ubuntu-latest)
- **Tests**: Unit tests (`cargo test`) and strict linting (`cargo clippy -D warnings`)
- **Purpose**: Ensure code quality before building (blocks build on test failures)

**2. Build Job** (depends on test job)
Compiles binaries for multiple platforms:
- **Platforms**:
  - Linux: x86_64-musl, i686-musl, arm-musl, armhf-musl, aarch64-musl
  - Windows: x86_64-msvc
  - macOS: x86_64, aarch64
  - FreeBSD: x86_64
- **Outputs**:
  - Raw binaries (uploaded as artifacts for all builds)
  - `.deb` packages (Linux targets only, release tags only)
  - `.tar.gz` tarballs (all platforms, release tags only)
- **Cross-compilation**: Uses `houseabsolute/actions-rust-cross` for cross-platform builds

**3. Repository Job** (release tags only, depends on test + build)
Manages Debian package repository:
- **Trigger**: Tags matching `v*` or `test-release*`
- **Actions**:
  - Downloads all `.deb` packages from build job
  - Uses `reprepro` to build/update Debian repository
  - Commits to `publish` branch (or `pre_publish` for test releases)

**4. Crate Job** (release tags only, depends on test + build)
Publishes to crates.io:
- **Trigger**: Tags matching `v*` or `test-release*`
- **Dry-run mode**: Test releases (`test-release*` tags) don't actually publish

### Release Process

1. **Development**: Push to feature branches triggers test + build jobs
2. **Pull Requests**: Full CI validation (test + build matrix)
3. **Test Release**: Tag with `test-release*` to simulate release process without publishing
4. **Production Release**: Tag with `v*` (e.g., `v0.1.6`) to:
   - Build and publish binaries to GitHub Releases (as draft)
   - Update Debian repository on `publish` branch
   - Publish crate to crates.io

### Tags and Branches

- **Ignored tags**: `quicssh-*` (reserved for upstream project)
- **Ignored branches**: `pre_publish`, `publish` (used for Debian repository)
- **Release tags**: `v*` (production), `test-release*` (testing)

### Binary Distribution

Users can install via:
- **GitHub Releases**: Download platform-specific `.tar.gz` or `.deb` from releases page
- **Debian Repository**: Add repository from `publish` branch and `apt install`
- **Cargo**: `cargo install quicssh-rs-robust` from crates.io
- **Manual Build**: Clone and `cargo build --release`
