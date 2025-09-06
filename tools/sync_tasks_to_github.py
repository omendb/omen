#!/usr/bin/env python3
"""
Sync TASKS.md to GitHub Issues for human visibility.
Run weekly or when major milestones are reached.
"""

import re
import subprocess
from pathlib import Path


def parse_tasks_md():
    """Parse TASKS.md and extract issues to create."""
    tasks_file = Path(__file__).parent.parent / "TASKS.md"
    if not tasks_file.exists():
        print("TASKS.md not found")
        return []
    
    content = tasks_file.read_text()
    issues = []
    
    # Extract critical bugs
    bug_section = re.search(r'### 🚨 Critical(.*?)###', content, re.DOTALL)
    if bug_section:
        bugs = re.findall(r'\*\*BUG-(\d+)\*\*: (.*?)\n', bug_section.group(1))
        for bug_id, title in bugs:
            issues.append({
                'title': f'[BUG-{bug_id}] {title}',
                'body': f'Tracked in TASKS.md as BUG-{bug_id}\n\nSee TASKS.md for details.',
                'labels': 'bug,critical'
            })
    
    # Extract high priority tasks
    high_priority = re.search(r'#### High Priority(.*?)####', content, re.DOTALL)
    if high_priority:
        tasks = re.findall(r'- \[ \] \*\*(.*?)\*\*', high_priority.group(1))
        for task in tasks:
            issues.append({
                'title': f'[TASK] {task}',
                'body': f'High priority task from TASKS.md\n\n{task}',
                'labels': 'enhancement,high-priority'
            })
    
    return issues


def create_github_issue(title, body, labels):
    """Create a GitHub issue using gh CLI."""
    # Check if issue already exists
    check_cmd = f'gh issue list --search "{title}" --limit 1 --json title'
    result = subprocess.run(check_cmd, shell=True, capture_output=True, text=True)
    
    if result.stdout.strip() != '[]':
        print(f"Issue already exists: {title}")
        return
    
    # Create new issue
    cmd = f'''gh issue create --title "{title}" --body "{body}" --label "{labels}"'''
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    
    if result.returncode == 0:
        print(f"Created issue: {title}")
    else:
        print(f"Failed to create issue: {title}")
        print(result.stderr)


def sync_to_github():
    """Main sync function."""
    print("Syncing TASKS.md to GitHub Issues...")
    
    # Check if gh CLI is installed
    if subprocess.run("which gh", shell=True, capture_output=True).returncode != 0:
        print("Error: gh CLI not installed. Install with: brew install gh")
        return
    
    issues = parse_tasks_md()
    
    if not issues:
        print("No critical issues to sync")
        return
    
    print(f"Found {len(issues)} issues to sync")
    
    for issue in issues[:5]:  # Limit to 5 to avoid spam
        create_github_issue(
            issue['title'],
            issue['body'],
            issue['labels']
        )
    
    print("\nSync complete! View issues at:")
    print("https://github.com/USERNAME/omendb/issues")


if __name__ == "__main__":
    # Dry run by default
    print("DRY RUN - Would create these issues:")
    issues = parse_tasks_md()
    for i in issues[:5]:
        print(f"  - {i['title']}")
    
    print("\nTo actually create issues, run:")
    print("  python tools/sync_tasks_to_github.py --create")