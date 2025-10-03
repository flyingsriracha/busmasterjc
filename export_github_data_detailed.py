#!/usr/bin/env python3
"""
Export all issues and pull requests from GitHub repository WITH FULL COMMENT THREADS
This version digs into each issue to get all responses and solutions from users.

Usage:
    python3 export_github_data_detailed.py
    
IMPORTANT: Set GITHUB_TOKEN environment variable to avoid rate limits!
    export GITHUB_TOKEN=your_token_here
    python3 export_github_data_detailed.py
"""

import json
import csv
import os
import sys
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError
import time
from datetime import datetime

REPO_OWNER = "rbei-etas"
REPO_NAME = "busmaster"
OUTPUT_DIR = "github_export_detailed"

def make_request(url):
    """Make GitHub API request with optional authentication"""
    headers = {
        'Accept': 'application/vnd.github.v3+json',
        'User-Agent': 'Python-GitHub-Exporter'
    }
    
    # Add token if available
    token = os.environ.get('GITHUB_TOKEN')
    if token:
        headers['Authorization'] = f'token {token}'
    
    req = Request(url, headers=headers)
    
    try:
        with urlopen(req) as response:
            return json.loads(response.read().decode())
    except HTTPError as e:
        if e.code == 403:
            print(f"\n❌ Rate limit exceeded. Error: {e}")
            print("Set GITHUB_TOKEN environment variable to increase rate limit.")
            print("Without token: 60 requests/hour")
            print("With token: 5000 requests/hour")
            sys.exit(1)
        elif e.code == 404:
            print(f"Warning: 404 Not Found for {url}")
            return None
        raise

def get_all_issues(include_prs=True):
    """Fetch all issues (and optionally PRs)"""
    items = []
    page = 1
    per_page = 100
    max_page = 10  # Can fetch up to 1000 issues
    
    print(f"Fetching issues and pull requests...")
    
    while page <= max_page:
        url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/issues?state=all&page={page}&per_page={per_page}&sort=created&direction=desc"
        print(f"  Page {page}...", end=' ', flush=True)
        
        try:
            data = make_request(url)
        except HTTPError as e:
            if e.code == 422:
                print(f"Reached pagination limit")
                break
            raise
        
        if not data:
            print("Done!")
            break
            
        items.extend(data)
        print(f"Got {len(data)} items (total: {len(items)})")
        
        if len(data) < per_page:
            print("Done!")
            break
            
        page += 1
        time.sleep(0.5)  # Be nice to the API
    
    # Separate issues and PRs
    if include_prs:
        return items
    else:
        return [item for item in items if 'pull_request' not in item]

def get_issue_comments(issue_number):
    """Fetch all comments for a specific issue"""
    url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/issues/{issue_number}/comments"
    comments = []
    
    try:
        data = make_request(url)
        if data:
            comments = data
    except Exception as e:
        print(f"Error fetching comments for issue #{issue_number}: {e}")
    
    time.sleep(0.3)  # Rate limiting
    return comments

def enrich_issue_with_comments(issue):
    """Add full comment thread to issue data"""
    issue_number = issue.get('number')
    comments_count = issue.get('comments', 0)
    
    # Create enriched issue object
    enriched = {
        'number': issue_number,
        'title': issue.get('title', ''),
        'state': issue.get('state'),
        'created_at': issue.get('created_at'),
        'updated_at': issue.get('updated_at'),
        'closed_at': issue.get('closed_at', ''),
        'author': issue.get('user', {}).get('login', ''),
        'author_url': issue.get('user', {}).get('html_url', ''),
        'labels': [label['name'] for label in issue.get('labels', [])],
        'url': issue.get('html_url'),
        'body': issue.get('body', ''),
        'body_text': issue.get('body_text', issue.get('body', '')),
        'comments_count': comments_count,
        'is_pull_request': 'pull_request' in issue,
        'comments': []
    }
    
    # Fetch comments if any exist
    if comments_count > 0:
        print(f"  Issue #{issue_number}: Fetching {comments_count} comments...", end=' ', flush=True)
        comments = get_issue_comments(issue_number)
        
        for comment in comments:
            enriched['comments'].append({
                'id': comment.get('id'),
                'author': comment.get('user', {}).get('login', ''),
                'author_url': comment.get('user', {}).get('html_url', ''),
                'created_at': comment.get('created_at'),
                'updated_at': comment.get('updated_at'),
                'body': comment.get('body', ''),
                'url': comment.get('html_url')
            })
        
        print(f"✓ Got {len(comments)} comments")
    
    return enriched

def export_to_json(data, filename):
    """Export data to JSON file"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    with open(filepath, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    print(f"✓ Saved {len(data)} items to {filepath}")

def export_to_csv(data, filename):
    """Export data to CSV file with comment summaries"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    
    if not data:
        print(f"No data to export to {filepath}")
        return
    
    fieldnames = ['number', 'title', 'state', 'created_at', 'updated_at', 
                  'closed_at', 'author', 'labels', 'url', 'comments_count',
                  'body_preview', 'has_comments', 'comment_authors']
    
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        
        for item in data:
            # Get unique comment authors
            comment_authors = list(set([c['author'] for c in item.get('comments', [])]))
            
            row = {
                'number': item.get('number'),
                'title': item.get('title', '').replace('\n', ' ').replace('\r', ''),
                'state': item.get('state'),
                'created_at': item.get('created_at'),
                'updated_at': item.get('updated_at'),
                'closed_at': item.get('closed_at', ''),
                'author': item.get('author', ''),
                'labels': ';'.join(item.get('labels', [])),
                'url': item.get('url'),
                'comments_count': item.get('comments_count', 0),
                'body_preview': (item.get('body', '') or '')[:200].replace('\n', ' ').replace('\r', ''),
                'has_comments': 'Yes' if item.get('comments_count', 0) > 0 else 'No',
                'comment_authors': ';'.join(comment_authors)
            }
            
            writer.writerow(row)
    
    print(f"✓ Saved {len(data)} items to {filepath}")

def export_conversation_csv(data, filename):
    """Export a flattened CSV showing each comment as a separate row"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    
    fieldnames = ['issue_number', 'issue_title', 'issue_state', 'issue_url',
                  'comment_type', 'author', 'created_at', 'text_preview']
    
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        
        for item in data:
            # Write the original issue/post
            writer.writerow({
                'issue_number': item.get('number'),
                'issue_title': item.get('title', '').replace('\n', ' '),
                'issue_state': item.get('state'),
                'issue_url': item.get('url'),
                'comment_type': 'ISSUE',
                'author': item.get('author', ''),
                'created_at': item.get('created_at'),
                'text_preview': (item.get('body', '') or '')[:500].replace('\n', ' ').replace('\r', ' ')
            })
            
            # Write each comment
            for comment in item.get('comments', []):
                writer.writerow({
                    'issue_number': item.get('number'),
                    'issue_title': item.get('title', '').replace('\n', ' '),
                    'issue_state': item.get('state'),
                    'issue_url': item.get('url'),
                    'comment_type': 'COMMENT',
                    'author': comment.get('author', ''),
                    'created_at': comment.get('created_at'),
                    'text_preview': (comment.get('body', '') or '')[:500].replace('\n', ' ').replace('\r', ' ')
                })
    
    print(f"✓ Saved conversation view to {filepath}")

def main():
    # Create output directory
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    print("=" * 80)
    print(f"BUSMASTER GitHub Issues & Comments Detailed Exporter")
    print(f"Repository: {REPO_OWNER}/{REPO_NAME}")
    print(f"Output directory: {OUTPUT_DIR}/")
    print("=" * 80)
    
    # Check rate limit
    token = os.environ.get('GITHUB_TOKEN')
    if token:
        print("✓ Using GitHub token (5000 requests/hour limit)")
    else:
        print("⚠ WARNING: No GitHub token found!")
        print("  You're limited to 60 requests/hour without authentication.")
        print("  This script needs to fetch each issue + comments separately.")
        print("  For 650+ issues, you NEED a token!")
        print("\n  To get a token:")
        print("  1. Go to https://github.com/settings/tokens")
        print("  2. Generate new token (classic) with 'public_repo' scope")
        print("  3. export GITHUB_TOKEN=your_token_here")
        print("\n  Press Ctrl+C to cancel, or Enter to continue anyway...")
        input()
    
    print("-" * 80)
    
    # Step 1: Fetch all issues
    print("\n[Step 1/3] Fetching all issues...")
    all_items = get_all_issues()
    
    # Separate issues and PRs
    issues = [item for item in all_items if 'pull_request' not in item]
    prs = [item for item in all_items if 'pull_request' in item]
    
    print(f"\n✓ Found {len(issues)} issues and {len(prs)} pull requests")
    
    # Step 2: Enrich issues with comments
    print(f"\n[Step 2/3] Fetching comments for {len(issues)} issues...")
    print("This may take several minutes depending on number of comments...")
    
    enriched_issues = []
    for i, issue in enumerate(issues, 1):
        print(f"\n[{i}/{len(issues)}] Issue #{issue['number']}: {issue['title'][:60]}")
        enriched = enrich_issue_with_comments(issue)
        enriched_issues.append(enriched)
        
        # Progress update every 50 issues
        if i % 50 == 0:
            print(f"\n--- Progress: {i}/{len(issues)} issues processed ---")
    
    # Step 3: Export everything
    print(f"\n[Step 3/3] Exporting data...")
    
    # Export full JSON with all comments
    export_to_json(enriched_issues, 'issues_with_comments.json')
    
    # Export CSV summaries
    export_to_csv(enriched_issues, 'issues_summary.csv')
    export_conversation_csv(enriched_issues, 'issues_conversations.csv')
    
    # Create detailed summary
    total_comments = sum(len(issue.get('comments', [])) for issue in enriched_issues)
    issues_with_comments = len([i for i in enriched_issues if i.get('comments_count', 0) > 0])
    
    summary = {
        'repository': f"{REPO_OWNER}/{REPO_NAME}",
        'export_date': datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
        'total_issues': len(enriched_issues),
        'open_issues': len([i for i in enriched_issues if i['state'] == 'open']),
        'closed_issues': len([i for i in enriched_issues if i['state'] == 'closed']),
        'issues_with_comments': issues_with_comments,
        'total_comments_fetched': total_comments,
        'total_prs': len(prs)
    }
    
    export_to_json(summary, 'export_summary.json')
    
    print("\n" + "=" * 80)
    print("EXPORT COMPLETE!")
    print("=" * 80)
    print(f"  Total Issues: {summary['total_issues']}")
    print(f"    - Open: {summary['open_issues']}")
    print(f"    - Closed: {summary['closed_issues']}")
    print(f"  Issues with Comments: {issues_with_comments}")
    print(f"  Total Comments Fetched: {total_comments}")
    print("=" * 80)
    print("\nFiles created:")
    print(f"  📄 {OUTPUT_DIR}/issues_with_comments.json")
    print(f"     → Full JSON with ALL issue details and comment threads")
    print(f"  📊 {OUTPUT_DIR}/issues_summary.csv")
    print(f"     → CSV summary with issue metadata and comment counts")
    print(f"  💬 {OUTPUT_DIR}/issues_conversations.csv")
    print(f"     → Flattened CSV showing each comment as a row (good for analysis)")
    print(f"  📋 {OUTPUT_DIR}/export_summary.json")
    print(f"     → Export statistics")
    print("\n✅ All data exported successfully!")
    print("\nNow you have:")
    print("  ✓ All issues with their full descriptions")
    print("  ✓ All comments and responses from users")
    print("  ✓ Solutions and discussions for each problem")
    print("  ✓ Ready for modernization planning!")

if __name__ == '__main__':
    main()

