# PRD: Git Branch Indicator in Prompt Input

## Overview

Add a visual indicator showing the current Git branch name in the prompt input's
dock tray, positioned immediately after the auto-accept permissions (shield) button.

This feature is implemented as a **Tauri plugin command** for the desktop app.

---

## Problem Statement

Users working on multiple branches need quick visibility of which Git branch they're
currently on without leaving the OpenCode interface or running terminal commands.

---

## Goals

- Provide instant visibility of the current Git branch
- Maintain visual consistency with existing dock tray elements
- Desktop-only feature via Tauri command
- Only display when the project is a Git repository

---

## Non-Goals

- Branch switching functionality from the UI
- Git status indicators (dirty/clean state)
- Multiple repository support in a single view
- Real-time branch change detection (fetches on directory change)

---

## User Stories

1. As a developer, I want to see which Git branch I'm on so I can ensure I'm
   working in the correct context

---

## Technical Approach

### Architecture

The feature is implemented as a Tauri plugin command:

- **Rust Command**: `get_git_branch` in `src-tauri/src/git.rs`
- **Platform Interface**: `getGitBranch` in `Platform` type
- **Desktop Implementation**: Uses Tauri bindings
- **Web**: Not available (graceful fallback)

### Implementation

#### 1. Rust Command (src-tauri/src/git.rs)

```rust
#[derive(Clone, serde::Serialize, specta::Type, Debug)]
pub struct GitBranchInfo {
    pub branch: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn get_git_branch(_app: AppHandle, directory: String) -> Result<GitBranchInfo, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&directory)
        .output()
        .map_err(|e| format!("Failed to execute git command: {e}"))?;

    if !output.status.success() {
        return Ok(GitBranchInfo { branch: None });
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(GitBranchInfo {
        branch: if branch.is_empty() { None } else { Some(branch) },
    })
}
```

#### 2. Platform Interface (packages/app/src/context/platform.tsx)

```typescript
export type Platform = {
  // ... existing fields
  getGitBranch?(directory: string): Promise<string | null>
}
```

#### 3. Desktop Implementation (packages/desktop/src/index.tsx)

```typescript
async getGitBranch(directory: string) {
  const result = await commands.getGitBranch(directory).catch(() => null)
  return result?.branch ?? null
}
```

#### 4. UI Component (prompt-input.tsx)

```tsx
const [branch, setBranch] = createSignal<string | undefined>()

createEffect(
  on(
    () => sdk.directory,
    (dir) => {
      if (!platform.getGitBranch) return
      platform.getGitBranch(dir).then((result) => {
        setBranch(result ?? undefined)
      })
    },
  ),
)

// In render:
<Show when={branch()}>
  <Tooltip placement="top" gutter={8} value={language.t("prompt.branch.tooltip")}>
    <div class="flex items-center gap-1 px-2 h-7 text-13-regular text-text-dimmed">
      <Icon name="branch" size="small" />
      <span class="truncate max-w-[120px]">{branch()}</span>
    </div>
  </Tooltip>
</Show>
```

#### 5. Internationalization

Add to all language files in `packages/app/src/i18n/`:

```typescript
"prompt.branch.tooltip": "Current Git branch"
```

### Files Modified

| File                                           | Change Type              |
| ---------------------------------------------- | ------------------------ |
| `packages/desktop/src-tauri/src/git.rs`        | New Rust module          |
| `packages/desktop/src-tauri/src/lib.rs`        | Register git module      |
| `packages/desktop/src/bindings.ts`             | Auto-generated bindings  |
| `packages/desktop/src/index.tsx`               | Platform implementation  |
| `packages/app/src/context/platform.tsx`        | Add getGitBranch to type |
| `packages/app/src/components/prompt-input.tsx` | Add branch state and UI  |
| `packages/app/src/i18n/*.ts`                   | Add tooltip translations |

---

## UI/UX Design

### Visual Layout

```
+-------------------------------------------------------------------------+
| [Agent v] [Model v] [Variant v] [shield] [branch feature/branch-name]   |
|                                  |        |                             |
|                                  |        +-- NEW INDICATOR             |
|                                  +-- Auto-accept button                 |
+-------------------------------------------------------------------------+
```

### Specifications

| Property   | Value                              |
| ---------- | ---------------------------------- |
| Icon       | `branch` (existing in icon set)    |
| Text style | `text-13-regular text-text-dimmed` |
| Max width  | 120px (truncated with ellipsis)    |
| Height     | 28px (h-7, matches other elements) |
| Padding    | 8px horizontal (px-2)              |

### Behavior

- Hidden when project is not a Git repository
- Hidden on web platform (desktop only)
- Fetches branch on directory change
- Tooltip shows "Current Git branch" on hover
- Branch name truncates gracefully for long names

---

## Testing Strategy

### Manual Testing

1. Open OpenCode Desktop in a Git repository - Branch should display
2. Open OpenCode Desktop in a non-Git directory - No branch indicator
3. Open OpenCode Web - No branch indicator (expected)
4. Long branch name - Truncates with ellipsis, full name in tooltip

### Edge Cases

- Detached HEAD state (shows commit hash)
- No Git installed on system
- Git repository with no commits yet

---

## Rollout Plan

1. Implement feature in `feature/git-current-branch-indicator` branch
2. Manual QA testing
3. Create PR targeting `dev` branch
4. Code review
5. Merge and release

---

## Installation

There are several strategies to install and enable the Git branch indicator feature.

### Strategy 1: Build from source

Clone the repository and build the desktop app:

```bash
git clone https://github.com/sst/opencode.git
cd opencode
bun install
bun run --filter opencode-desktop build
```

The built app will be in `packages/desktop/src-tauri/target/release/`.

### Strategy 2: Development mode

For local development and testing:

```bash
git clone https://github.com/sst/opencode.git
cd opencode
bun install
bun run --filter opencode-desktop dev
```

This launches the app with hot-reload enabled.

### Strategy 3: Pre-built releases

Once merged and released, download the pre-built binaries from the
[GitHub Releases](https://github.com/sst/opencode/releases) page.

Available formats:

- **macOS**: `.dmg` (Universal)
- **Windows**: `.msi` or `.exe`
- **Linux**: `.AppImage`, `.deb`, or `.rpm`

### Strategy 4: Package managers

After release, install via package managers:

```bash
# macOS (Homebrew)
brew install --cask opencode

# Windows (winget)
winget install opencode

# Linux (snap)
snap install opencode
```

### Verify installation

Once installed, open the app in a Git repository. The branch indicator appears
in the prompt input's dock tray, after the shield button.

---

## Success Metrics

- Feature works without errors
- No performance degradation
- Positive user feedback

---

## Future Enhancements (Out of Scope)

- Click to copy branch name
- Color indicator for dirty/clean state
- Branch switching from UI
- Show ahead/behind commit count
- Real-time branch change detection via file watcher
