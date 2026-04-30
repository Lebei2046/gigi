# Step-by-Step Summary: Using Git Worktree from Zero

## Prerequisite: Verify Git Installation
Ensure Git is installed on your system:
```bash
git --version
```

## Step 1: Create a Test Repository
1. **Create and enter a directory**:
   ```bash
   mkdir git-worktree-demo && cd git-worktree-demo
   ```
2. **Initialize Git**:
   ```bash
   git init
   ```
3. **Add an initial file and commit**:
   ```bash
   echo "Initial content" > README.md && git add README.md && git commit -m "Initial commit"
   ```

## Step 2: Add a New Worktree (with New Branch)
Use `-b` to create the branch during worktree setup (avoids the "invalid reference" error):
```bash
git worktree add -b feature/new-feature ../feature-branch
```
- `../feature-branch`: Path to the new worktree directory
- `-b feature/new-feature`: Creates the branch if it doesn’t exist

## Step 3: Make Changes in Both Worktrees
1. **In the feature worktree** (`../feature-branch`):
   ```bash
   cd ../feature-branch && echo "Feature code" > feature.txt && git add feature.txt && git commit -m "Add feature"
   ```
2. **In the main worktree** (`git-worktree-demo`):
   ```bash
   cd ../git-worktree-demo && echo "Main code" > main.txt && git add main.txt && git commit -m "Add main"
   ```

## Step 4: List All Worktrees
Verify both worktrees are active:
```bash
git worktree list
```

## Step 5: Merge Changes (Optional)
Merge the feature branch into main:
```bash
cd git-worktree-demo && git merge feature/new-feature
```

## Step 6: Remove the Worktree
Clean up the feature worktree:
```bash
git worktree remove ../feature-branch
```

## Step 7: Prune Unused Worktrees
Clean up leftover metadata:
```bash
git worktree prune
```

## Step 8: Verify Cleanup
Confirm only the main worktree remains:
```bash
git worktree list
```

### Key Benefits
- **Parallel Development**: Work on multiple branches simultaneously
- **Space Efficiency**: Shares the Git object database (no duplicate clones)
- **Isolation**: Changes in one worktree don’t affect others
