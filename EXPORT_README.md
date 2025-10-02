# Exporting BUSMASTER GitHub Issues & Pull Requests

This directory contains scripts to export all issues and pull requests from the BUSMASTER repository for analysis.

## Quick Start (Python - Recommended)

**No installation required!** Just run:

```bash
python3 export_github_data.py
```

This will create a `github_export/` folder with:
- `issues.json` - Full issue data (650+ issues)
- `issues.csv` - Summary for Excel/Google Sheets
- `pull_requests.json` - Full PR data
- `pull_requests.csv` - PR summary
- `summary.json` - Statistics

### Rate Limits

**Without authentication:** 60 requests/hour (might not be enough for 650+ issues)

**With GitHub token:** 5,000 requests/hour (recommended)

To use a token:
```bash
export GITHUB_TOKEN=your_github_token_here
python3 export_github_data.py
```

#### Creating a GitHub Token:
1. Go to https://github.com/settings/tokens
2. Click "Generate new token (classic)"
3. Give it a name (e.g., "BUSMASTER Export")
4. Select scopes: `public_repo` (for public repositories)
5. Click "Generate token"
6. Copy the token and use it above

---

## Alternative: Using GitHub CLI

If you prefer the GitHub CLI:

### 1. Install GitHub CLI:
```bash
brew install gh
```

### 2. Run the export script:
```bash
./export_github_data.sh
```

This will authenticate you if needed and export everything.

---

## What You'll Get

### Issues CSV Format:
| Number | Title | State | Created | Updated | Author | Labels | URL | Comments |
|--------|-------|-------|---------|---------|--------|--------|-----|----------|
| 1329 | "Driver selection failed" problem | open | ... | ... | ... | ... | ... | ... |

### Analysis Ideas:

Once exported, you can:

1. **Identify patterns** - Sort by labels, creation date, or author
2. **Categorize issues** - Group by driver type, component, or severity  
3. **Track trends** - See which issues are oldest, most commented, etc.
4. **Import to tools** - Use CSV in Excel, Google Sheets, Jupyter notebooks, etc.
5. **Create dashboards** - Visualize issue distribution and trends

### Example Analysis (Python):

```python
import pandas as pd

# Load the data
issues = pd.read_csv('github_export/issues.csv')

# Most common labels
label_counts = issues['labels'].str.split(';').explode().value_counts()
print(label_counts)

# Issues by state
print(issues['state'].value_counts())

# Oldest open issues
oldest = issues[issues['state'] == 'open'].sort_values('created_at').head(10)
print(oldest[['number', 'title', 'created_at']])
```

---

## Troubleshooting

**Error: Rate limit exceeded**
- Solution: Use a GitHub token (see above)

**Error: Permission denied**
- Solution: Run `chmod +x export_github_data.py`

**Error: No module named 'pandas'**
- Only needed for analysis, not for export
- Install: `pip3 install pandas`

---

## Next Steps

After exporting:

1. **Open `issues.csv` in Excel/Google Sheets** for quick browsing
2. **Search for specific keywords** (e.g., "driver", "crash", "compilation")
3. **Create a prioritized list** of issues to tackle
4. **Analyze trends** to understand pain points
5. **Plan improvements** based on data-driven insights

Good luck with your BUSMASTER fork! 🚀

