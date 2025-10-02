#!/usr/bin/env python3
"""
Export all issues and pull requests from GitHub repository
No authentication required for public repos (but rate limited to 60 requests/hour)
With authentication: 5000 requests/hour

Usage:
    python3 export_github_data.py
    
Optional: Set GITHUB_TOKEN environment variable for higher rate limits
    export GITHUB_TOKEN=your_token_here
    python3 export_github_data.py
"""

import json
import csv
import os
import sys
from urllib.request import Request, urlopen
from urllib.error import HTTPError, URLError
import time

REPO_OWNER = "rbei-etas"
REPO_NAME = "busmaster"
OUTPUT_DIR = "github_export"

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
            print(f"Rate limit exceeded. Error: {e}")
            print("Set GITHUB_TOKEN environment variable to increase rate limit.")
            sys.exit(1)
        raise

def get_all_items(item_type):
    """Fetch all issues or pull requests (item_type: 'issues' or 'pulls')"""
    items = []
    page = 1
    per_page = 100
    max_page = 10  # GitHub API limits to ~1000 items via pagination
    
    print(f"Fetching {item_type}...")
    
    while page <= max_page:
        url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/{item_type}?state=all&page={page}&per_page={per_page}&sort=created&direction=desc"
        print(f"  Page {page}...", end=' ')
        
        try:
            data = make_request(url)
        except HTTPError as e:
            if e.code == 422:
                print(f"Reached pagination limit at page {page}")
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
        time.sleep(0.75)  # Be nice to the API
    
    if page > max_page:
        print(f"Note: Stopped at {max_page} pages due to GitHub API limits")
        print(f"To get ALL items, use the GitHub CLI method or provide a GITHUB_TOKEN")
    
    return items

def export_to_json(data, filename):
    """Export data to JSON file"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    with open(filepath, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    print(f"Saved {len(data)} items to {filepath}")

def export_to_csv(data, filename, is_pr=False):
    """Export data to CSV file"""
    filepath = os.path.join(OUTPUT_DIR, filename)
    
    if not data:
        print(f"No data to export to {filepath}")
        return
    
    fieldnames = ['number', 'title', 'state', 'created_at', 'updated_at', 
                  'closed_at', 'author', 'labels', 'url', 'comments_count']
    
    if is_pr:
        fieldnames.insert(6, 'merged_at')
    
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        
        for item in data:
            row = {
                'number': item.get('number'),
                'title': item.get('title', '').replace('\n', ' ').replace('\r', ''),
                'state': item.get('state'),
                'created_at': item.get('created_at'),
                'updated_at': item.get('updated_at'),
                'closed_at': item.get('closed_at', ''),
                'author': item.get('user', {}).get('login', ''),
                'labels': ';'.join([label['name'] for label in item.get('labels', [])]),
                'url': item.get('html_url'),
                'comments_count': item.get('comments', 0)
            }
            
            if is_pr:
                row['merged_at'] = item.get('merged_at', '')
            
            writer.writerow(row)
    
    print(f"Saved {len(data)} items to {filepath}")

def main():
    # Create output directory
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    print(f"Exporting data from {REPO_OWNER}/{REPO_NAME}")
    print(f"Output directory: {OUTPUT_DIR}/")
    print("-" * 60)
    
    # Check rate limit
    token = os.environ.get('GITHUB_TOKEN')
    if token:
        print("✓ Using GitHub token (higher rate limit)")
    else:
        print("⚠ No GitHub token found (limited to 60 requests/hour)")
        print("  To increase limit: export GITHUB_TOKEN=your_token_here")
    print("-" * 60)
    
    # Fetch issues (this includes PRs in GitHub API)
    all_issues = get_all_items('issues')
    
    # Separate issues and PRs
    issues = [item for item in all_issues if 'pull_request' not in item]
    prs_basic = [item for item in all_issues if 'pull_request' in item]
    
    print(f"\nFound {len(issues)} issues and {len(prs_basic)} pull requests")
    
    # Export issues
    print("\n--- Exporting Issues ---")
    export_to_json(issues, 'issues.json')
    export_to_csv(issues, 'issues.csv', is_pr=False)
    
    # Export PRs
    print("\n--- Exporting Pull Requests ---")
    export_to_json(prs_basic, 'pull_requests.json')
    export_to_csv(prs_basic, 'pull_requests.csv', is_pr=True)
    
    # Create summary
    summary = {
        'repository': f"{REPO_OWNER}/{REPO_NAME}",
        'export_date': time.strftime('%Y-%m-%d %H:%M:%S'),
        'total_issues': len(issues),
        'open_issues': len([i for i in issues if i['state'] == 'open']),
        'closed_issues': len([i for i in issues if i['state'] == 'closed']),
        'total_prs': len(prs_basic),
        'open_prs': len([p for p in prs_basic if p['state'] == 'open']),
        'closed_prs': len([p for p in prs_basic if p['state'] == 'closed']),
        'merged_prs': len([p for p in prs_basic if p.get('merged_at')])
    }
    
    export_to_json(summary, 'summary.json')
    
    print("\n" + "=" * 60)
    print("Export Summary:")
    print(f"  Total Issues: {summary['total_issues']} (Open: {summary['open_issues']}, Closed: {summary['closed_issues']})")
    print(f"  Total PRs: {summary['total_prs']} (Open: {summary['open_prs']}, Merged: {summary['merged_prs']})")
    print("=" * 60)
    print("\nFiles created:")
    print(f"  📄 {OUTPUT_DIR}/issues.json - Full issue data")
    print(f"  📊 {OUTPUT_DIR}/issues.csv - Issue summary (Excel-friendly)")
    print(f"  📄 {OUTPUT_DIR}/pull_requests.json - Full PR data")
    print(f"  📊 {OUTPUT_DIR}/pull_requests.csv - PR summary (Excel-friendly)")
    print(f"  📋 {OUTPUT_DIR}/summary.json - Export summary")
    print("\n✅ Export complete!")

if __name__ == '__main__':
    main()

