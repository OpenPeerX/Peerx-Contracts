# Changelog

All notable changes to the PeerX CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-20

### Added
- Initial release of PeerX CLI
- `health` subcommand for pre-flight health checks
- Comprehensive health checks:
  - RPC endpoint reachability
  - Contract existence verification
  - Contract pause status monitoring
  - Admin address verification
  - Oracle data freshness checks
- Multiple output formats: JSON, YAML, and human-readable
- Exit codes: 0 (healthy), 1 (warning), 2 (critical)
- Flexible configuration via:
  - Command-line arguments
  - Environment variables
  - Configuration file (~/.peerx/config.json)
- Detailed, structured output with timing information
- Color-coded terminal output for better readability
- Verbose and quiet modes
- Configurable timeouts and thresholds
- Comprehensive documentation and examples

### Features
- **Observability**: Full visibility into contract and infrastructure health
- **DX (Developer Experience)**: Easy to use CLI with intuitive commands
- **Automation-ready**: Structured JSON output and exit codes for CI/CD
- **Extensible**: Modular architecture for adding future commands

### Documentation
- Complete README with usage examples
- Configuration guide
- Troubleshooting section
- CI/CD integration examples
- Kubernetes probe examples

[0.1.0]: https://github.com/coderolisa/Peerx-Contracts/releases/tag/v0.1.0
