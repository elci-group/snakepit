# GitHub Upload Instructions

Follow these steps to upload Snakepit to GitHub.

## Prerequisites

- GitHub account (https://github.com)
- Git installed locally
- SSH key configured (optional but recommended)

## Step 1: Create GitHub Repository

1. Go to https://github.com/new
2. Repository name: `snakepit`
3. Description: `A dynamic Rust-based Python dependency installer with shell integration`
4. Choose public or private
5. **Do NOT** initialize with README, .gitignore, or license (we already have them)
6. Click "Create repository"

## Step 2: Initialize Local Git Repository

```bash
cd /home/adminx/snakepit

# Initialize git if not already done
git init

# Configure user (if not set globally)
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Or use global config:
# git config --global user.name "Your Name"
# git config --global user.email "your.email@example.com"
```

## Step 3: Add Remote and Push

### Using HTTPS (no SSH setup needed)

```bash
# Add remote
git remote add origin https://github.com/YOUR_USERNAME/snakepit.git

# Verify
git remote -v

# Stage all files
git add .

# Initial commit
git commit -m "feat: Initial Snakepit release with shell integration

- Multi-backend Python dependency management (pip, conda, poetry)
- Virtual environment management
- Bash shell integration with auto-install on import errors
- Daemon process monitoring
- Beautiful CLI with colorized output
- Cross-platform support (Linux, macOS, Windows)

See README.md for full feature list and usage instructions."

# Rename branch to main (if needed)
git branch -M main

# Push to GitHub
git push -u origin main
```

### Using SSH (recommended for security)

```bash
# Add remote using SSH
git remote add origin git@github.com:YOUR_USERNAME/snakepit.git

# Rest is the same as HTTPS method
git add .
git commit -m "feat: Initial Snakepit release with shell integration"
git branch -M main
git push -u origin main
```

## Step 4: Verify Upload

```bash
# Check status
git status

# View remote
git remote -v

# View commits
git log --oneline
```

## Step 5: GitHub Repository Configuration (Optional but Recommended)

### Settings > General

- Add description: "A dynamic Rust-based Python dependency installer"
- Add topics: `python`, `dependency-manager`, `rust`, `cli`, `bash-integration`
- Enable "Issues"
- Enable "Discussions"
- Enable "Projects"

### Settings > Code and automation > Actions

- Verify CI workflows are enabled
- Check workflow runs in Actions tab

### Settings > Branches

- Set main as default branch
- Add branch protection rule for main (optional):
  - Require pull request reviews
  - Require status checks to pass
  - Dismiss stale review approvals

## Step 6: Create Release (Optional)

```bash
# Create a tag
git tag -a v0.1.0 -m "Initial release: Snakepit 0.1.0"

# Push tag
git push origin v0.1.0
```

Then go to GitHub repository and create release from the tag.

## Step 7: Add Repository Links

Update these files if they reference your repository:

- `INSTALLATION.md` - Lines 16, 20 (repository URLs)
- `CONTRIBUTING.md` - Line 20 (repository URL)
- `SECURITY.md` - Update email address
- Any CI/CD badges in README

## Troubleshooting

### "fatal: not a git repository"

```bash
# Initialize git
git init
```

### "Repository not found"

- Verify repository exists on GitHub
- Check username and repository name spelling
- Verify authentication (SSH key or HTTPS token)

### "Permission denied (publickey)"

- SSH key not configured: Use HTTPS instead
- Or set up SSH key:
  ```bash
  ssh-keygen -t ed25519 -C "your.email@example.com"
  cat ~/.ssh/id_ed25519.pub  # Copy this
  # Add to https://github.com/settings/keys
  ```

### "branch 'main' set up to track remote 'origin/main'"

This is normal - your local main branch now tracks the remote.

## After Upload

1. Create first issue as a tracking mechanism
2. Create discussions for feature requests
3. Check CI workflow runs
4. Monitor notifications for activity
5. Update SECURITY.md with actual email contact

## Next Steps

1. Share repository link
2. Add stars and watchers
3. Set up GitHub Pages for documentation (optional)
4. Configure branch protection rules
5. Plan first minor release (v0.1.1, v0.2.0, etc.)

## Quick Command Reference

```bash
# Check status
git status

# Add changes
git add .

# Commit
git commit -m "commit message"

# Push to main branch
git push origin main

# Pull latest changes
git pull origin main

# View commit history
git log --oneline -10

# Create new branch
git checkout -b feature/new-feature

# Push new branch
git push -u origin feature/new-feature

# Delete local branch
git branch -d feature/old-feature

# Delete remote branch
git push origin --delete feature/old-feature
```

## Resources

- [GitHub Docs](https://docs.github.com/)
- [Git Handbook](https://guides.github.com/introduction/git-handbook/)
- [Commit Message Best Practices](https://chris.beams.io/posts/git-commit/)

---

Ready to upload? Use the commands in **Step 2-3** above!
