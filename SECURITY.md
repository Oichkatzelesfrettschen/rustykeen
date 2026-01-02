# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it by:

1. **Do not** open a public issue
2. Email the maintainer directly (if available) or use GitHub's private vulnerability reporting

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 7 days and work to address the issue promptly.

## Security Practices

This project follows these security practices:

- `unsafe_code = "forbid"` in all crates except `kenken-simd`
- `warnings = "deny"` workspace-wide
- Dependency updates reviewed regularly
- CI enforces clippy and formatting checks
