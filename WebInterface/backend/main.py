"""FastAPI application exposing the GitHub export via a modern web API."""
from __future__ import annotations

from collections import Counter
from typing import Any, Dict, List, Optional

from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware

from .data_access import (
    Issue,
    PullRequest,
    load_issues,
    load_pull_requests,
    load_summary,
    refresh_cache,
)

app = FastAPI(
    title="BUSMASTER Modernization API",
    description=(
        "REST API that exposes the GitHub issues and pull requests from the "
        "exported data set. The endpoints power the prototype web interface "
        "used to browse modernization work."
    ),
    version="1.0.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ---------------------------------------------------------------------------
# Serialization helpers
# ---------------------------------------------------------------------------

def _issue_to_dict(issue: Issue) -> Dict[str, Any]:
    return {
        "number": issue.number,
        "title": issue.title,
        "body": issue.body,
        "state": issue.state,
        "labels": issue.labels,
        "created_at": issue.created_at.isoformat() if issue.created_at else None,
        "updated_at": issue.updated_at.isoformat() if issue.updated_at else None,
        "closed_at": issue.closed_at.isoformat() if issue.closed_at else None,
        "comments": issue.comments,
    }


def _pr_to_dict(pr: PullRequest) -> Dict[str, Any]:
    return {
        "number": pr.number,
        "title": pr.title,
        "body": pr.body,
        "state": pr.state,
        "created_at": pr.created_at.isoformat() if pr.created_at else None,
        "updated_at": pr.updated_at.isoformat() if pr.updated_at else None,
        "closed_at": pr.closed_at.isoformat() if pr.closed_at else None,
        "merged_at": pr.merged_at.isoformat() if pr.merged_at else None,
    }


# ---------------------------------------------------------------------------
# Query helpers
# ---------------------------------------------------------------------------

def _apply_issue_filters(
    issues: List[Issue],
    state: Optional[str],
    label: Optional[str],
    search: Optional[str],
) -> List[Issue]:
    filtered = issues
    if state and state.lower() != "all":
        filtered = [issue for issue in filtered if issue.state.lower() == state.lower()]
    if label:
        label_lower = label.lower()
        filtered = [
            issue
            for issue in filtered
            if any(lbl.lower() == label_lower for lbl in issue.labels)
        ]
    if search:
        search_lower = search.lower()
        filtered = [
            issue
            for issue in filtered
            if search_lower in issue.title.lower() or search_lower in issue.body.lower()
        ]
    return filtered


def _apply_pr_filters(
    prs: List[PullRequest],
    state: Optional[str],
    search: Optional[str],
) -> List[PullRequest]:
    filtered = prs
    if state and state.lower() != "all":
        filtered = [pr for pr in filtered if pr.state.lower() == state.lower()]
    if search:
        search_lower = search.lower()
        filtered = [
            pr
            for pr in filtered
            if search_lower in pr.title.lower() or search_lower in pr.body.lower()
        ]
    return filtered


def _paginate(items: List[Any], page: int, per_page: int) -> List[Any]:
    if per_page <= 0:
        raise ValueError("per_page must be positive")
    start = (page - 1) * per_page
    end = start + per_page
    return items[start:end]


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------

@app.get("/", tags=["health"])
def root() -> Dict[str, str]:
    return {"status": "ok", "message": "BUSMASTER modernization API is ready"}


@app.get("/api/issues", tags=["issues"])
def list_issues(
    state: Optional[str] = Query("open", description="Filter issues by state (open/closed/all)"),
    label: Optional[str] = Query(
        None, description="Filter issues by exact label name"
    ),
    search: Optional[str] = Query(None, description="Search issue titles and bodies"),
    page: int = Query(1, ge=1),
    per_page: int = Query(25, ge=1, le=100),
) -> Dict[str, Any]:
    issues = load_issues()
    filtered = _apply_issue_filters(issues, state=state, label=label, search=search)
    total = len(filtered)
    paginated = _paginate(filtered, page=page, per_page=per_page)
    return {
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": [_issue_to_dict(issue) for issue in paginated],
    }


@app.get("/api/issues/labels", tags=["issues"])
def list_issue_labels() -> Dict[str, List[str]]:
    labels = sorted({label for issue in load_issues() for label in issue.labels})
    return {"labels": labels}


@app.get("/api/issues/{number}", tags=["issues"])
def get_issue(number: int) -> Dict[str, Any]:
    for issue in load_issues():
        if issue.number == number:
            return _issue_to_dict(issue)
    raise HTTPException(status_code=404, detail=f"Issue #{number} not found")


@app.get("/api/pull_requests", tags=["pull_requests"])
def list_pull_requests(
    state: Optional[str] = Query("all", description="Filter pull requests by state"),
    search: Optional[str] = Query(None, description="Search PR titles and bodies"),
    page: int = Query(1, ge=1),
    per_page: int = Query(25, ge=1, le=100),
) -> Dict[str, Any]:
    prs = load_pull_requests()
    filtered = _apply_pr_filters(prs, state=state, search=search)
    total = len(filtered)
    paginated = _paginate(filtered, page=page, per_page=per_page)
    return {
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": [_pr_to_dict(pr) for pr in paginated],
    }


@app.get("/api/pull_requests/{number}", tags=["pull_requests"])
def get_pull_request(number: int) -> Dict[str, Any]:
    for pr in load_pull_requests():
        if pr.number == number:
            return _pr_to_dict(pr)
    raise HTTPException(status_code=404, detail=f"Pull request #{number} not found")


@app.get("/api/stats", tags=["stats"])
def get_stats() -> Dict[str, Any]:
    issues = load_issues()
    prs = load_pull_requests()
    summary = load_summary()

    issue_states = Counter(issue.state.lower() for issue in issues)
    pr_states = Counter(pr.state.lower() for pr in prs)

    label_counter = Counter()
    for issue in issues:
        label_counter.update(label.lower() for label in issue.labels)

    top_labels = [
        {"label": label, "count": count}
        for label, count in label_counter.most_common(10)
    ]

    merged_count = sum(1 for pr in prs if pr.merged_at is not None)

    return {
        "repository": summary.get("repository"),
        "export_date": summary.get("export_date"),
        "issues": {
            "total": len(issues),
            "open": issue_states.get("open", 0),
            "closed": issue_states.get("closed", 0),
        },
        "pull_requests": {
            "total": len(prs),
            "open": pr_states.get("open", 0),
            "closed": pr_states.get("closed", 0),
            "merged": merged_count,
        },
        "top_labels": top_labels,
    }


@app.post("/api/cache/refresh", tags=["maintenance"])
def refresh() -> Dict[str, str]:
    refresh_cache()
    # Force reload so that any IO errors bubble up immediately.
    load_issues()
    load_pull_requests()
    return {"status": "reloaded"}
