# Regular Expression Examples for --src-path

The `--src-path` parameter accepts regular expressions to match file paths within ZIP archives. This provides flexible filtering of conda packages based on their location and naming patterns.

**Important**: When multiple files match the regex pattern, only the first matching conda package will be processed. This ensures predictable behavior and prevents accidental processing of duplicate or unwanted packages.

## Basic Patterns

### Simple Directory Matching
```bash
# Match files in the artifacts/ directory
--src-path "^artifacts/"

# Match files in conda-packages/ directory and subdirectories
--src-path "^conda-packages/"
```

### Platform-Specific Directories
```bash
# Match Linux 64-bit packages only
--src-path "^artifacts/linux-64/"

# Match multiple platforms
--src-path "^artifacts/(linux-64|osx-64|win-64)/"

# Match any platform directory pattern
--src-path "^artifacts/[a-z]+-[0-9]+/"
```

## Advanced Patterns

### Build Directory Patterns
```bash
# Match numbered build directories
--src-path "^build-[0-9]+/conda/"

# Match specific build number ranges
--src-path "^build-(123|456|789)/conda/"

# Match any build directory structure
--src-path "^build-\d+/conda/.*"
```

### File Extension Filtering
```bash
# Match only .conda files (anywhere in archive)
--src-path ".*\.conda$"

# Match both .conda and .tar.bz2 files
--src-path ".*\.(conda|tar\.bz2)$"

# Match conda files in specific directory
--src-path "^artifacts/linux-64/.*\.conda$"
```

### Package Name Patterns
```bash
# Match packages with 'python' in the name
--src-path "python.*\.(conda|tar\.bz2)$"

# Match packages for specific Python versions
--src-path ".*-py3[0-9]_.*\.(conda|tar\.bz2)$"

# Match packages with version numbers
--src-path ".*-[0-9]+\.[0-9]+.*\.(conda|tar\.bz2)$"
```

## Complex Examples

### CI/CD Artifact Patterns
```bash
# Azure DevOps build artifacts
--src-path "^artifact_[0-9]+/conda/(linux-64|osx-64)/.*\.conda$"

# GitHub Actions artifacts
--src-path "^conda-packages/.*-py[0-9]+.*\.(conda|tar\.bz2)$"

# Jenkins build pattern
--src-path "^workspace/build-[0-9]+/output/conda/.*$"
```

### Multi-Level Directory Matching
```bash
# Match packages in any platform subdirectory (first match only)
--src-path "^.*/conda/(linux-64|osx-64|win-64)/.*\.(conda|tar\.bz2)$"

# Match packages with specific architecture (first match only)
--src-path ".*/linux-64/.*\.(conda|tar\.bz2)$"

# Skip test directories (first non-test match only)
--src-path "^(?!.*test).*\.(conda|tar\.bz2)$"
```

## Real-World Examples

### Example 1: Azure DevOps Pipeline
```bash
# Pipeline artifacts in specific build
meso-forge-mirror mirror \
  --src "https://dev.azure.com/org/proj/_apis/build/builds/1234/artifacts" \
  --src-type zip-url \
  --src-path "^drop/conda/linux-64/.*\.conda$" \
  --tgt ./local-repo
```

### Example 2: GitHub Release Assets
```bash
# Release assets with platform filtering
meso-forge-mirror mirror \
  --src "https://github.com/owner/repo/releases/download/v1.0/packages.zip" \
  --src-type zip-url \
  --src-path "^conda-packages/(linux-64|osx-64)/.*\.(conda|tar\.bz2)$" \
  --tgt ./local-repo
```

### Example 3: Build Artifact Archive
```bash
# Multiple build outputs (first match only)
meso-forge-mirror mirror \
  --src "build-outputs.zip" \
  --src-type zip \
  --src-path "^(build-[0-9]+|staging)/conda/.*\.conda$" \
  --tgt ./local-repo
```

## Pattern Testing Tips

### Common Regex Elements
- `^` - Start of string/path
- `$` - End of string/path  
- `.` - Any single character
- `.*` - Any characters (zero or more)
- `\.` - Literal dot (escaped)
- `\d` - Any digit (0-9)
- `[0-9]` - Any digit (alternative to \d)
- `[a-z]` - Any lowercase letter
- `(option1|option2)` - Alternative matching
- `+` - One or more of the preceding element
- `*` - Zero or more of the preceding element
- `?` - Zero or one of the preceding element

### Escaping Special Characters
When using regex patterns in shell commands, remember to:
- Use quotes around the pattern: `"^artifacts/.*"`
- Escape backslashes in shell: `"^artifacts\\linux-64\\.*\\.conda$"`
- Use single quotes to avoid shell interpretation: `'^artifacts/.*\.conda$'`

### Testing Your Patterns
1. Start with simple patterns and add complexity gradually
2. Test with known file structures first
3. Use online regex testers with sample file paths
4. Check the error messages for pattern validation feedback
5. Remember that only the first match will be processed - make patterns specific enough to match the desired file

### Why First Match Only?

The first-match behavior ensures:
- **Predictable results**: Same pattern always processes the same file
- **Performance optimization**: No need to scan entire archive after finding a match
- **Avoiding duplicates**: Prevents accidentally processing multiple versions of the same package
- **Clear intent**: Forces users to write specific patterns rather than broad ones

## First Match Behavior Details

When a regex pattern matches multiple conda packages, only the first match encountered will be processed:

### Example Scenario
```
# ZIP Archive contents (in order):
1. README.md
2. artifacts/linux-64/package-1.0-py39_0.conda  ← FIRST MATCH
3. artifacts/linux-64/package-2.0-py39_0.conda  ← SKIPPED
4. artifacts/osx-64/package-1.0-py39_0.conda    ← SKIPPED
5. build/logs/output.txt

# Pattern: ^artifacts/.*\.conda$
# Result: Only file #2 will be mirrored
```

### Controlling Which File Matches First

Since ZIP file order determines matching, use specific patterns:

**Too broad** (may match unwanted files):
```bash
--src-path "^artifacts/.*"  # Matches many files
```

**Better** (targets specific package):
```bash
--src-path "^artifacts/linux-64/package-1\.0.*\.conda$"
--src-path "^artifacts/osx-64/.*\.conda$"  # For macOS packages
--src-path "^artifacts/.*/mypackage-.*\.conda$"  # Specific package name
```

### Pattern Strategies

1. **Version-specific patterns**:
   ```bash
   --src-path "^artifacts/.*-2\.1\.0-.*\.conda$"
   ```

2. **Platform-specific patterns**:
   ```bash
   --src-path "^artifacts/linux-64/.*\.conda$"
   ```

3. **Package name patterns**:
   ```bash
   --src-path "^artifacts/.*/python-.*\.conda$"
   ```

4. **Build-specific patterns**:
   ```bash
   --src-path "^build-123/conda/.*\.conda$"
   ```

To ensure you get the specific package you want:
- Use more specific patterns that target exactly one file
- Include version numbers, platform names, or specific package names
- Test patterns against known archive structures
- Remember that file order in ZIP determines precedence

## Error Messages

When no packages match your pattern, you'll see helpful output:
```
Error: No conda packages found in ZIP file matching pattern: '^wrong-path/.*'

All files in ZIP:
  1: artifacts/linux-64/package-1.0.conda
  2: artifacts/osx-64/package-1.0.conda
  3: build/output.log

Hint: File paths must match regex pattern '^wrong-path/.*' and have .conda or .tar.bz2 extensions
```

This helps you adjust your pattern based on the actual file structure in the archive.