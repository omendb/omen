# GitHub Integration with Claude Code

## Setup Instructions

### 1. Install the Claude GitHub App
Visit https://github.com/apps/claude-code and install the app on your ZenDB repository.

### 2. Add API Key to Repository Secrets
1. Go to Settings → Secrets and variables → Actions
2. Add a new repository secret named `ANTHROPIC_API_KEY`
3. Set the value to your Anthropic API key

### 3. Workflow Configuration
The Claude Code workflow is configured in `.github/workflows/claude-code.yml` with the following features:

## Usage

### In Issues
Mention `@claude` in an issue comment to get assistance:
```
@claude Can you help me understand how the MVCC implementation should work?
```

### In Pull Requests
- Mention `@claude` in PR comments for code review
- Automatic code review runs on all PR changes to Rust files
- Claude will analyze code quality, performance, security, and suggest improvements

### Supported Triggers
- Issue comments containing `@claude`
- PR review comments containing `@claude`
- New issues with `@claude` in the body
- New or updated PRs with `@claude` in the description

## Features

### Automatic Code Review
On every pull request, Claude will:
1. Check Rust best practices
2. Identify potential bugs
3. Analyze performance implications
4. Review security considerations
5. Suggest improvements

### Interactive Assistance
Claude can help with:
- Answering technical questions
- Debugging issues
- Explaining code behavior
- Suggesting implementation approaches
- Reviewing architecture decisions

## Workflow Details

The workflow uses two main jobs:

1. **claude-assist**: Responds to `@claude` mentions in issues and PRs
2. **code-review**: Automatically reviews code changes in pull requests

Both jobs require the `ANTHROPIC_API_KEY` secret to be configured in your repository settings.