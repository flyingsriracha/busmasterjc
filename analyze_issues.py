#!/usr/bin/env python3
"""
Quick analysis of BUSMASTER GitHub issues
Usage: python3 analyze_issues.py
"""

import json
import csv
from collections import Counter, defaultdict
from datetime import datetime

# Load data
with open('github_export/issues.csv', 'r', encoding='utf-8') as f:
    reader = csv.DictReader(f)
    issues = list(reader)

with open('github_export/summary.json', 'r') as f:
    summary = json.load(f)

print("=" * 80)
print("BUSMASTER ISSUES ANALYSIS")
print("=" * 80)
print(f"\nExport Date: {summary['export_date']}")
print(f"Repository: {summary['repository']}")
print(f"\n{'─' * 80}")

# Basic stats
print("\n📊 OVERALL STATISTICS")
print(f"{'─' * 80}")
print(f"Total Issues: {summary['total_issues']}")
print(f"  ├─ Open:   {summary['open_issues']} ({summary['open_issues']/summary['total_issues']*100:.1f}%)")
print(f"  └─ Closed: {summary['closed_issues']} ({summary['closed_issues']/summary['total_issues']*100:.1f}%)")
print(f"\nTotal Pull Requests: {summary['total_prs']}")
print(f"  ├─ Open:   {summary['open_prs']}")
print(f"  ├─ Merged: {summary['merged_prs']}")
print(f"  └─ Closed: {summary['closed_prs']}")

# Most active contributors
print(f"\n{'─' * 80}")
print("\n👥 TOP ISSUE REPORTERS")
print(f"{'─' * 80}")
authors = Counter([issue['author'] for issue in issues])
for author, count in authors.most_common(10):
    print(f"  {author:30} {count:3} issues")

# Label analysis
print(f"\n{'─' * 80}")
print("\n🏷️  MOST COMMON LABELS")
print(f"{'─' * 80}")
all_labels = []
for issue in issues:
    if issue['labels']:
        all_labels.extend(issue['labels'].split(';'))
label_counts = Counter(all_labels)
if label_counts:
    for label, count in label_counts.most_common(15):
        if label:  # Skip empty labels
            print(f"  {label:40} {count:3} issues")
else:
    print("  No labels found")

# Find issues by keyword
print(f"\n{'─' * 80}")
print("\n🔍 ISSUES BY TOPIC (Keyword Search)")
print(f"{'─' * 80}")

keywords = {
    'driver': 'Driver/Hardware',
    'crash|exception|error': 'Crashes/Errors',
    'build|compile|compilation': 'Build Issues',
    'node|simulation': 'Node Simulation',
    'can|lin': 'CAN/LIN Protocol',
    'test|automation': 'Test Automation',
    'ui|interface|window': 'UI/UX',
    'install|uninstall': 'Installation'
}

import re

for pattern, description in keywords.items():
    count = 0
    for issue in issues:
        if re.search(pattern, issue['title'].lower(), re.IGNORECASE):
            count += 1
    print(f"  {description:25} {count:3} issues")

# Recent issues (last 50)
print(f"\n{'─' * 80}")
print("\n🆕 MOST RECENT OPEN ISSUES (Top 20)")
print(f"{'─' * 80}")

open_issues = [i for i in issues if i['state'] == 'open']
# Sort by number (descending = most recent)
recent = sorted(open_issues, key=lambda x: int(x['number']), reverse=True)[:20]

for issue in recent:
    number = issue['number']
    title = issue['title'][:60]
    created = issue['created_at'][:10] if issue['created_at'] else 'N/A'
    comments = issue.get('comments_count', 0)
    print(f"  #{number:4} | {created} | 💬{comments:2} | {title}")

# Oldest open issues
print(f"\n{'─' * 80}")
print("\n⏰ OLDEST OPEN ISSUES (Top 20)")
print(f"{'─' * 80}")

oldest = sorted(open_issues, key=lambda x: int(x['number']))[:20]

for issue in oldest:
    number = issue['number']
    title = issue['title'][:60]
    created = issue['created_at'][:10] if issue['created_at'] else 'N/A'
    comments = issue.get('comments_count', 0)
    print(f"  #{number:4} | {created} | 💬{comments:2} | {title}")

# Most discussed issues
print(f"\n{'─' * 80}")
print("\n💬 MOST DISCUSSED OPEN ISSUES (Top 15)")
print(f"{'─' * 80}")

discussed = sorted(open_issues, key=lambda x: int(x.get('comments_count', 0)), reverse=True)[:15]

for issue in discussed:
    number = issue['number']
    title = issue['title'][:55]
    created = issue['created_at'][:10] if issue['created_at'] else 'N/A'
    comments = issue.get('comments_count', 0)
    state = '🟢' if issue['state'] == 'open' else '🔴'
    print(f"  #{number:4} | {state} | {created} | 💬{comments:3} | {title}")

# Keyword-based categorization for priority planning
print(f"\n{'─' * 80}")
print("\n🎯 PRIORITY ISSUES BY CATEGORY (Open Issues Only)")
print(f"{'─' * 80}")

categories = {
    'Critical Crashes': ['crash', 'exception', 'freeze', 'hang'],
    'Driver Problems': ['driver', 'hardware', 'selection failed', 'connection'],
    'Build/Compilation': ['build', 'compile', 'compilation', 'cmake', 'visual studio'],
    'Node Simulation': ['node', 'simulation', 'dll', 'handler'],
    'Test Issues': ['test', 'automation', 'executor'],
    'UI/Display': ['ui', 'window', 'display', 'font', 'interface']
}

for category, keywords_list in categories.items():
    matching = []
    for issue in open_issues:
        title_lower = issue['title'].lower()
        if any(kw in title_lower for kw in keywords_list):
            matching.append(issue)
    
    if matching:
        print(f"\n  {category} ({len(matching)} open issues):")
        for issue in matching[:5]:  # Show top 5
            number = issue['number']
            title = issue['title'][:60]
            print(f"    #{number:4} - {title}")

print(f"\n{'=' * 80}")
print("\n💡 SUGGESTIONS:")
print("  1. Open 'github_export/issues.csv' in Excel for detailed browsing")
print("  2. Focus on oldest open issues - they're likely foundational problems")
print("  3. Driver issues are most common - consider standardizing the DIL")
print("  4. Many build issues suggest documentation/toolchain improvements needed")
print(f"\n{'=' * 80}\n")

