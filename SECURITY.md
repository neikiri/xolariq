# Security Policy

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Yes             |

## Reporting a vulnerability

If you discover a security vulnerability in Xolariq, **please do not open a public issue.**

Instead, report it privately via email:

📧 **dev@neiki.eu**

Please include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce the issue or a proof-of-concept.
- The version of Xolariq you are using.

I will acknowledge your report within **48 hours** and aim to provide a fix or mitigation plan within **7 days**, depending on severity.

## Scope

Xolariq is a local-only desktop application — it does not make network requests, host servers, or process remote input. Security concerns most likely involve:

- **Installer integrity** — tampered MSI/EXE bundles.
- **Sidecar binaries** — compromised FFmpeg, pandoc or 7-Zip executables.
- **Shell extension** — privilege escalation via the COM handler or registry entries.
- **File path handling** — path traversal or injection through crafted filenames.

## Disclosure policy

- Vulnerabilities will be patched in a new release as soon as possible.
- Credit will be given to the reporter in the changelog unless they prefer to remain anonymous.
