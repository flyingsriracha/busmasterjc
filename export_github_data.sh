#!/bin/bash

# Export all issues and pull requests from BUSMASTER repository
# Usage: ./export_github_data.sh

REPO="rbei-etas/busmaster"
OUTPUT_DIR="github_export"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Authenticating with GitHub..."
gh auth status || gh auth login

echo "Exporting all issues..."
gh issue list --repo "$REPO" --state all --limit 10000 --json number,title,state,createdAt,updatedAt,closedAt,author,labels,body,comments > "$OUTPUT_DIR/issues.json"

echo "Exporting all pull requests..."
gh pr list --repo "$REPO" --state all --limit 10000 --json number,title,state,createdAt,updatedAt,closedAt,mergedAt,author,labels,body,comments,reviews > "$OUTPUT_DIR/pull_requests.json"

echo "Converting to CSV for easier viewing..."
# Convert issues to CSV
jq -r '["Number","Title","State","Created","Updated","Author","Labels"], (.[] | [.number, .title, .state, .createdAt, .updatedAt, .author.login, (.labels | map(.name) | join(";"))]) | @csv' "$OUTPUT_DIR/issues.json" > "$OUTPUT_DIR/issues.csv"

# Convert PRs to CSV
jq -r '["Number","Title","State","Created","Updated","Merged","Author","Labels"], (.[] | [.number, .title, .state, .createdAt, .updatedAt, .mergedAt, .author.login, (.labels | map(.name) | join(";"))]) | @csv' "$OUTPUT_DIR/pull_requests.json" > "$OUTPUT_DIR/pull_requests.csv"

echo "Export complete! Files saved in $OUTPUT_DIR/"
echo "- issues.json (full data)"
echo "- issues.csv (summary)"
echo "- pull_requests.json (full data)"
echo "- pull_requests.csv (summary)"

