# Axios npm Supply Chain Attack — March 31, 2026

## Summary

On March 31, 2026, the popular JavaScript HTTP client **axios** (100M+ weekly npm downloads) suffered a **supply chain attack**. Two malicious versions — `1.14.1` and `0.30.4` — were published to npm after attackers compromised the account of maintainer **Jason Saayman** (`@jasonsaayman`).

The malicious versions were live for approximately **3 hours** (00:21–03:29 UTC) before npm removed them. During that window, any system running `npm install` could have been compromised.

## What Happened

### Account Compromise

- The attacker compromised `@jasonsaayman`'s machine via a social engineering attack (masquerading as a well-known company wanting to "collaborate on the project").
- The attacker stole browser session cookies/credentials, changed the npm account email to an anonymous ProtonMail address (`ifstap@proton.me`), and used existing npm tokens to publish malicious versions — bypassing the normal GitHub Actions CI/CD pipeline.
- `@jasonsaayman` had 2FA/MFA enabled, but the attacker used stored browser passwords to bypass 2FA for email changes and leveraged an existing npm token (from the v0.x branch) to publish.

### Attack Mechanism

The attacker **did not modify any axios source code**. Instead:

1. Added a hidden malicious dependency `plain-crypto-js@4.2.1` to `package.json`.
2. `plain-crypto-js` had a `postinstall` hook that ran `setup.js`.
3. The dropper used **double-layer obfuscation** (reversed Base64 + XOR cipher with key `OrDeR_7077`).
4. It detected the host OS and downloaded platform-specific RAT (Remote Access Trojan) payloads from a C2 server at `sfrclak[.]com:8000` (IP: `142.11.206.73`).
5. After execution, the malware **self-destructed** — deleting `setup.js`, removing the malicious `package.json`, and replacing it with a clean version to avoid detection.

### Platform-Specific Payloads

| Platform | Payload | Location |
|----------|---------|----------|
| **macOS** | AppleScript-based binary RAT | `/Library/Caches/com.apple.act.mond` |
| **Windows** | PowerShell RAT (masquerading as Windows Terminal) | `%PROGRAMDATA%\wt.exe` |
| **Linux** | Python RAT | `/tmp/ld.py` |

The RAT supported commands for arbitrary code execution, binary injection, filesystem enumeration, and process termination.

## Timeline (UTC)

| Time | Event |
|------|-------|
| 2026-03-30 ~06:00 | `plain-crypto-js@4.2.0` (clean version) published to create registry history |
| 2026-03-30 23:59 | `plain-crypto-js@4.2.1` (malicious) published |
| 2026-03-31 00:21 | `axios@1.14.1` published with malicious dependency |
| 2026-03-31 ~00:27 | Socket's scanner detects malicious version (~6 min) |
| 2026-03-31 01:00 | `axios@0.30.4` published with malicious dependency |
| 2026-03-31 ~03:15 | npm administration revokes all compromised tokens |
| 2026-03-31 03:29 | Both malicious axios versions removed from npm |

## Resolution

- Axios collaborator `@DigitalBrainJS` identified the compromise, pinned the GitHub issue, and contacted npm administration.
- npm removed the malicious versions and revoked all compromised tokens.
- The malicious `plain-crypto-js` package was replaced with a security placeholder on npm.
- All axios versions **except** `1.14.1` and `0.30.4` are safe.

## How to Check If You Were Affected

### 1. Check lockfiles for affected versions

```bash
# npm
grep -E '"axios"' package-lock.json | grep -E '1\.14\.1|0\.30\.4'

# yarn
grep -E 'axios@' yarn.lock | grep -E '1\.14\.1|0\.30\.4'

# bun
grep -E 'axios' bun.lock | grep -E '1\.14\.1|0\.30\.4'
```

### 2. Check for the malicious dependency

```bash
npm ls plain-crypto-js
```

### 3. Check for IOCs (Indicators of Compromise)

| Platform | What to Look For |
|----------|------------------|
| macOS | `/Library/Caches/com.apple.act.mond` binary |
| Windows | `%PROGRAMDATA%\wt.exe` |
| Linux | `/tmp/ld.py` Python script |
| Network | Outbound connections to `sfrclak[.]com` / `142.11.206.73:8000` |

## Recommendations

### If NOT affected (precautionary)

1. **Pin dependencies** — avoid using `^` or `~` ranges for critical packages.
2. **Commit lockfiles** and use `npm ci` (not `npm install`) in CI.
3. **Set minimum release age**: `npm config set min-release-age 3` (days).
4. **Disable postinstall scripts in CI**: `npm ci --ignore-scripts`.
5. **Blocklist** `plain-crypto-js` in your security tooling.

### If affected (assume breach)

1. **Isolate** any systems that ran `npm install` during the 3-hour window.
2. **Rotate ALL secrets** — API keys, SSH keys, cloud credentials, npm/GitHub tokens. Revoke and reissue.
3. **Check logs** for outbound connections to `sfrclak[.]com` or `142.11.206.73`.
4. **Rebuild environments** from known-clean snapshots — do not attempt to clean compromised systems.
5. **Audit CI pipelines** for the March 31 UTC window.

## Related Issues and Advisories

- GitHub Issue: [axios/axios#10604](https://github.com/axios/axios/issues/10604)
- Snyk Advisory: [SNYK-JS-AXIOS-15850650](https://security.snyk.io/vuln/SNYK-JS-AXIOS-15850650)
- Additional compromised packages: `@qqbrowser/openclaw-qbot@0.0.130`, `@shadanai/openclaw` (versions `2026.3.31-1`, `2026.3.31-2`)

## Key Takeaways

- This attack demonstrated high operational sophistication: pre-staged clean dependency versions, double-obfuscated droppers, platform-specific RATs, and anti-forensic self-deletion.
- **Lockfile enforcement** and **`npm ci`** would have prevented this for projects with committed lockfiles.
- npm's trusted publishing (OIDC-based) — when properly configured without fallback tokens — would have prevented unauthorized publishes.
- Social engineering remains the primary vector for maintainer account compromises.
