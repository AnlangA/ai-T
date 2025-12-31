# Release Guide

This guide explains how to create releases and where to find built executables.

## Finding Executables from the Recent Release Workflow Run

If you recently triggered the release workflow manually and are looking for the executables:

### Option 1: Download from GitHub Actions (Current Method)

1. Go to the [Actions tab](https://github.com/AnlangA/ai-T/actions/workflows/release.yml)
2. Click on the most recent "Release" workflow run
3. Scroll down to the **Artifacts** section at the bottom of the page
4. Download:
   - `ai-translate-linux-x86_64.tar.gz` - Linux executable
   - `ai-translate-windows-x86_64.zip` - Windows executable

**Note**: These artifacts are available for 90 days from the workflow run date.

### Option 2: Draft Releases (After Workflow Update)

After the changes in this PR are merged, future manual workflow triggers will automatically create draft releases:

1. Go to the [Releases page](https://github.com/AnlangA/ai-T/releases)
2. Look for draft releases (they're only visible to repository owners/collaborators)
3. Download the attached artifacts
4. Optionally, publish the draft release to make it public

## Creating Official Releases

To create an official release with executables:

### Method 1: Using Git Tags (Recommended for Official Releases)

```bash
# Create and push a version tag
git tag v0.1.0
git push origin v0.1.0
```

This will automatically:
1. Trigger the release workflow
2. Build executables for Linux and Windows
3. Create a public GitHub release
4. Attach the executables to the release

### Method 2: Manual Workflow Trigger (For Testing/Development)

1. Go to the [Actions tab](https://github.com/AnlangA/ai-T/actions/workflows/release.yml)
2. Click "Run workflow" button
3. Select the branch to build from
4. Click "Run workflow"

This will:
1. Build executables for Linux and Windows
2. Create a **draft release** (after PR is merged)
3. Artifacts are also available in the workflow run

## Artifact Details

The workflow creates two artifacts:

| Artifact | Platform | Contents |
|----------|----------|----------|
| `ai-translate-linux-x86_64.tar.gz` | Linux x86_64 | Single binary: `ai-translate` |
| `ai-translate-windows-x86_64.zip` | Windows x86_64 | Single binary: `ai-translate.exe` |

*Note: Artifact sizes may vary depending on the build but are typically 20-25 MB.*

## Downloading and Running

### Linux

```bash
# Download and extract
tar xzf ai-translate-linux-x86_64.tar.gz

# Make executable (if needed)
chmod +x ai-translate

# Run
./ai-translate
```

### Windows

1. Extract the ZIP file
2. Double-click `ai-translate.exe` or run from command line:
   ```cmd
   ai-translate.exe
   ```

## Troubleshooting

### "I can't find the executables"

- **For manual workflow runs**: Check the Actions tab → workflow run → Artifacts section
- **For tag-based releases**: Check the Releases page
- **After PR merge**: Manual runs will create draft releases (visible to repo collaborators)

### "The artifacts expired"

Workflow artifacts expire after 90 days. To get the executables:
1. Trigger a new manual workflow run, OR
2. Create a new tag for a permanent release

### "I need a different platform"

Currently, only Linux x86_64 and Windows x86_64 are supported. To add more platforms:
1. Edit `.github/workflows/release.yml`
2. Add entries to the build matrix for other targets (e.g., macOS, ARM)
3. Ensure required dependencies are installed for each platform
