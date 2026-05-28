# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.x     | ✅        |

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Email **security@paubox.com** with:

- A description of the vulnerability and its potential impact
- The SDK version(s) affected
- Steps to reproduce or a proof-of-concept
- Any suggested mitigations

We aim to acknowledge receipt within **2 business days** and provide a resolution timeline within **7 business days**. We will coordinate disclosure with the reporter before publishing any fix and will credit reporters in the release notes unless they prefer to remain anonymous.

## Security notes for SDK users

### Credential handling
- Store your Paubox API key in environment variables or a secrets manager — never in source code or version control
- The SDK never logs API keys, request bodies, or response bodies
- Rotate your key immediately if you suspect it has been exposed

### HIPAA / PHI
- The SDK is designed for use in HIPAA-regulated environments
- It does not log, cache, or transmit Protected Health Information (PHI) beyond the Paubox API calls you initiate
- Do not include PHI in log statements, error messages, or telemetry in your own application code
- `PauboxError::Http { body, .. }` may contain raw response bodies — treat as sensitive and do not log without scrubbing
- Consult your compliance team regarding obligations as a Covered Entity or Business Associate

### TLS
- The SDK uses `reqwest` with the `rustls-tls` backend; modern TLS (1.2+) is enforced by `rustls`
- Certificate verification is always on — there is no option to disable it
- Do not patch dependencies or supply a custom transport that weakens certificate verification

### API key rotation
Rotate your API key periodically and immediately if:
- It may have been exposed in logs, error messages, or version control
- A team member with access leaves the organisation
- You observe unexpected API activity in your Paubox dashboard
