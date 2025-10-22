# Security & Dependency Management

This document describes the vulnerability scanning and automatic dependency update features for Snakepit.

## Features

### 1. Automated Vulnerability Scanning

**What it does:**
- Scans dependencies for known security vulnerabilities
- Checks for unmaintained or yanked crates
- Validates dependency licenses
- Detects multiple versions of the same dependency
- Runs automatically on push, pull requests, and daily at 2 AM UTC

**Tools used:**
- `cargo-audit`: Checks dependencies against RustSec Advisory Database
- `cargo-deny`: Comprehensive dependency policy enforcement
- GitHub's dependency review action

### 2. Automatic Dependency Updates

**What it does:**
- Automatically creates pull requests for dependency updates
- Updates both Rust dependencies and GitHub Actions
- Groups minor and patch updates together
- Runs weekly on Mondays at 9 AM

**Managed by:**
- GitHub Dependabot

## GitHub Workflows

### Security Audit Workflow (`.github/workflows/security.yml`)

Three jobs run in parallel:

1. **security-audit**
   - Installs and runs `cargo-audit`
   - Generates JSON and text reports
   - Uploads artifacts for review
   - Fails on any vulnerabilities

2. **cargo-deny**
   - Runs comprehensive checks:
     - License compliance
     - Banned dependencies
     - Security advisories
     - Source verification

3. **dependency-review** (PR only)
   - Reviews new dependencies in pull requests
   - Fails on moderate or higher severity issues

### Triggers

- Push to `main` branch
- Pull requests to `main` branch
- Daily at 2 AM UTC (cron schedule)
- Manual dispatch via GitHub UI

## Local Usage

### Quick Scan

Run the automated script:

```bash
./check-vulnerabilities.sh
```

This script will:
1. Install required tools (if not present)
2. Update the advisory database
3. Run cargo-audit
4. Run cargo-deny checks
5. Check for outdated dependencies
6. Generate reports in `security-reports/`

### Manual Commands

**Check for vulnerabilities:**
```bash
cargo audit
```

**Update dependencies (safe):**
```bash
cargo update
```

**Check for outdated dependencies:**
```bash
cargo outdated
```

**Run all cargo-deny checks:**
```bash
cargo deny check
```

**Run specific cargo-deny checks:**
```bash
cargo deny check advisories    # Security advisories
cargo deny check licenses       # License compliance
cargo deny check bans          # Banned dependencies
cargo deny check sources       # Source verification
```

## Configuration Files

### `deny.toml`

Configures cargo-deny behavior:

- **Advisories**: Denies vulnerabilities, warns on unmaintained/yanked crates
- **Licenses**: Allows MIT, Apache-2.0, BSD variants; denies GPL/AGPL
- **Bans**: Warns on multiple versions, wildcard dependencies
- **Sources**: Only allows crates.io registry

You can customize these settings by editing `deny.toml`.

### `.github/dependabot.yml`

Configures Dependabot:

- **Schedule**: Weekly updates on Mondays at 9 AM
- **Limits**: 
  - 10 open PRs for Rust dependencies
  - 5 open PRs for GitHub Actions
- **Grouping**: Minor and patch updates grouped together
- **Labels**: Automatic `dependencies` and language-specific labels

## Security Reports

Reports are generated in `security-reports/` directory:

- `audit_TIMESTAMP.json`: Machine-readable audit results
- `audit_TIMESTAMP.txt`: Human-readable audit results
- `outdated_TIMESTAMP.json`: Outdated dependencies report

These reports are excluded from version control (see `.gitignore`).

## Responding to Vulnerabilities

### When a vulnerability is found:

1. **Review the advisory**: Check the GitHub Actions summary or local report
2. **Assess impact**: Determine if the vulnerability affects your usage
3. **Update dependencies**:
   ```bash
   cargo update
   ```
4. **Test thoroughly**:
   ```bash
   cargo test
   cargo clippy
   ```
5. **Verify fix**:
   ```bash
   cargo audit
   ```

### If update breaks compatibility:

1. Check `CHANGELOG.md` of the updated crate
2. Look for migration guides
3. Update your code as needed
4. Consider pinning the version temporarily if unable to update immediately

### If no fix is available:

1. Check if the vulnerability applies to your use case
2. Consider alternative dependencies
3. Add the advisory ID to `deny.toml` ignore list (temporarily) with a comment explaining why
4. Track the issue and remove the ignore once fixed

## Dependabot Pull Requests

When Dependabot creates a PR:

1. **Review the changes**: Check what versions are being updated
2. **Check CI**: Ensure all tests pass
3. **Review changelogs**: Look for breaking changes
4. **Test locally** (for major updates):
   ```bash
   git fetch origin
   git checkout dependabot/cargo/...
   cargo build
   cargo test
   ```
5. **Merge**: If everything looks good, merge the PR

### Auto-merge setup (optional)

For patch updates, you can enable auto-merge:

1. Go to repository settings
2. Enable "Allow auto-merge"
3. Dependabot will auto-merge patch updates that pass CI

## Best Practices

1. **Run scans before releases**: Always run `./check-vulnerabilities.sh` before tagging a release
2. **Keep dependencies updated**: Don't let dependency updates accumulate
3. **Review Dependabot PRs promptly**: Security updates should be merged quickly
4. **Monitor the security workflow**: Check GitHub Actions for any failures
5. **Subscribe to RustSec advisories**: Stay informed about new vulnerabilities
6. **Document exceptions**: If ignoring advisories in `deny.toml`, always comment why

## Troubleshooting

### "cargo-audit not found"

Install it:
```bash
cargo install cargo-audit
```

### "cargo-deny not found"

Install it:
```bash
cargo install cargo-deny
```

### "cargo-outdated not found"

Install it:
```bash
cargo install cargo-outdated
```

### Workflow fails but local checks pass

- Ensure your local toolchain matches CI (stable Rust)
- Check that deny.toml is committed
- Verify .github/workflows files are present

### Too many Dependabot PRs

Adjust in `.github/dependabot.yml`:
- Lower `open-pull-requests-limit`
- Change schedule from `weekly` to `monthly`
- Add more dependencies to `ignore` list

## Additional Resources

- [RustSec Advisory Database](https://rustsec.org/)
- [cargo-audit documentation](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny documentation](https://embarkstudios.github.io/cargo-deny/)
- [Dependabot documentation](https://docs.github.com/en/code-security/dependabot)
- [GitHub Security Advisories](https://docs.github.com/en/code-security/security-advisories)
