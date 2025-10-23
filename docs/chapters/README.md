# Documentation Chapters

This directory contains comprehensive chapter-level documentation for specific features and integrations of meso-forge-mirror.

## Chapter Structure

Each chapter is written in AsciiDoc format for better structure, cross-referencing, and maintainability compared to Markdown.

### Available Chapters

| Chapter | Description | Target Audience |
|---------|-------------|-----------------|
| [`github-integration.adoc`](github-integration.adoc) | Complete guide to GitHub Actions artifacts integration | Users, CI/CD engineers |
| [`azure-devops-integration.adoc`](azure-devops-integration.adoc) | Complete guide to Azure DevOps build artifacts integration | Users, Enterprise developers |

## AsciiDoc Benefits

These chapters use AsciiDoc format for several advantages:

- **Better Structure**: Hierarchical sections with automatic numbering
- **Cross-References**: Easy linking between sections and documents
- **Rich Formatting**: Tables, callouts, code blocks with syntax highlighting
- **Professional Output**: Can generate PDF, HTML, and other formats
- **Maintainability**: Cleaner syntax for complex documentation

## Reading the Documentation

### Online (GitHub)
GitHub renders AsciiDoc files natively, so you can read them directly in the web interface.

### Local Reading
For the best reading experience locally:

1. **AsciiDoc Processor**: Install `asciidoctor` for full rendering
   ```bash
   gem install asciidoctor
   asciidoctor docs/chapters/github-integration.adoc
   ```

2. **VS Code**: Install the "AsciiDoc" extension for syntax highlighting and preview

3. **Plain Text**: AsciiDoc is readable as plain text even without processing

## Chapter Guidelines

When adding new chapters:

1. **Use Consistent Structure**:
   - Start with chapter title (`= Chapter Title`)
   - Include table of contents (`:toc:`)
   - Use hierarchical sections (`==`, `===`, `====`)

2. **Include Standard Sections**:
   - Overview
   - Prerequisites/Authentication
   - Basic Usage
   - Advanced Examples
   - Configuration
   - Troubleshooting
   - Integration notes

3. **Follow Conventions**:
   - Use code blocks with appropriate syntax highlighting
   - Include practical examples
   - Reference related chapters and main documentation
   - Add cross-links where appropriate

## Migration Status

These chapters replace older Markdown documentation:

- ✅ `GITHUB_USAGE.md` → `github-integration.adoc` (removed)
- ✅ `AZURE_DEVOPS_USAGE.md` → `azure-devops-integration.adoc` (removed)

The old Markdown files have been removed after successful migration to AsciiDoc format.

## Contributing

When contributing to these chapters:

1. Follow the existing structure and style
2. Test examples to ensure they work
3. Update cross-references when adding new sections
4. Consider the target audience for each chapter
5. Update the main documentation index if adding new chapters

## Related Documentation

- [`../index.adoc`](../index.adoc) - Main documentation index
- [`../operator-guide.adoc`](../operator-guide.adoc) - Complete operator guide
- [`../../README.md`](../../README.md) - Project overview and quick start
