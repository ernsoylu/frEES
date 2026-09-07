# Security Policy

`frees` provides client-side, zero-backend physical systems simulation and modeling. Because models, proprietary formulas, experimental datasets, and simulations execute entirely within the user's browser or local CLI environment without telemetry or external API calls, safeguarding the execution environment and client-side boundary is paramount.

---

## 1. Supported Versions

Security updates and patches are applied to the active `main` branch and the latest minor release tags:

| Version | Supported |
|---|---|
| `>= 0.1.0` | :white_check_mark: |
| `< 0.1.0` | :x: |

---

## 2. Reporting a Vulnerability

If you discover a security vulnerability in `frees` (including WebAssembly memory isolation escapes, denial-of-service/infinite-loop parser traps, supply-chain vulnerabilities, or unexpected network leakages that violate our client-only privacy policy):

1. **Do NOT file a public issue.**
2. Report the vulnerability privately through GitHub's [Private Vulnerability Reporting](https://github.com/ernsoylu/frees/security/advisories/new) feature on the repository.
3. Alternatively, contact the maintainers directly via email at `security@frees.dev` (or the repository owner's GitHub profile contact).

### What to Include

Please provide:
- A clear description of the vulnerability and its potential security impact.
- Exact reproduction steps, including sample `.frees` model input or web interaction sequence.
- Browser environment, operating system, and engine version where the issue was observed.
- Any proof-of-concept code or proposed remediation.

---

## 3. Response & Disclosure Process

- **Acknowledgment**: You will receive an initial response acknowledging receipt within 48 hours.
- **Assessment**: The maintainers will investigate, determine severity, and coordinate a fix in a private branch.
- **Remediation & Advisory**: Once a patch is verified, a release will be tagged and a public security advisory published detailing the issue, CVE (if applicable), and credit to the reporter.
