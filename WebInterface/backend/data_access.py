"""Utilities to load and query the GitHub export data for the web API."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from functools import lru_cache
from pathlib import Path
from typing import Any, Dict, List, Optional

import csv
import json

ROOT = Path(__file__).resolve().parents[2]
EXPORT_DIR = ROOT / "github_export"


@dataclass(slots=True)
class Issue:
    """Lightweight representation of a GitHub issue."""

    number: int
    title: str
    body: str
    state: str
    labels: List[str]
    created_at: Optional[datetime]
    updated_at: Optional[datetime]
    closed_at: Optional[datetime]
    comments: int

    @property
    def is_open(self) -> bool:
        return self.state.lower() == "open"


@dataclass(slots=True)
class PullRequest:
    """Representation of a GitHub pull request."""

    number: int
    title: str
    body: str
    state: str
    created_at: Optional[datetime]
    updated_at: Optional[datetime]
    closed_at: Optional[datetime]
    merged_at: Optional[datetime]

    @property
    def is_open(self) -> bool:
        return self.state.lower() == "open"


def _parse_datetime(value: Optional[str]) -> Optional[datetime]:
    if not value:
        return None
    value = value.strip()
    if not value:
        return None
    # GitHub exports use ISO 8601 with a trailing Z for UTC.
    value = value.rstrip("Z")
    try:
        return datetime.fromisoformat(value)
    except ValueError:
        return None


def _load_json(path: Path) -> List[dict]:
    try:
        with path.open(encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError:
        return []
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Failed to parse JSON export '{path}': {exc}") from exc
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        return [data]
    raise RuntimeError(f"Unexpected JSON structure in '{path}'; expected a list or dict")


def _load_csv(path: Path) -> List[dict]:
    try:
        with path.open(encoding="utf-8", newline="") as handle:
            reader = csv.DictReader(handle)
            return list(reader)
    except FileNotFoundError:
        return []


def _load_issue_dicts() -> List[dict]:
    json_records = _load_json(EXPORT_DIR / "issues.json")
    if json_records:
        return json_records
    return _load_csv(EXPORT_DIR / "issues.csv")


def _load_pr_dicts() -> List[dict]:
    json_records = _load_json(EXPORT_DIR / "pull_requests.json")
    if json_records:
        return json_records
    return _load_csv(EXPORT_DIR / "pull_requests.csv")


@lru_cache(maxsize=1)
def load_issues() -> List[Issue]:
    records = _load_issue_dicts()
    issues: List[Issue] = []
    for record in records:
        labels_field = record.get("labels") or record.get("labels\n") or ""
        if isinstance(labels_field, str):
            labels = [label.strip() for label in labels_field.split(";") if label.strip()]
        elif isinstance(labels_field, list):
            labels = [str(label) for label in labels_field]
        else:
            labels = []
        issues.append(
            Issue(
                number=int(record.get("number") or record.get("id") or 0),
                title=(record.get("title") or "").strip(),
                body=(record.get("body") or "").strip(),
                state=(record.get("state") or "unknown").strip(),
                labels=labels,
                created_at=_parse_datetime(record.get("created_at")),
                updated_at=_parse_datetime(record.get("updated_at")),
                closed_at=_parse_datetime(record.get("closed_at")),
                comments=int(record.get("comments") or record.get("comments_count") or 0),
            )
        )
    return issues


@lru_cache(maxsize=1)
def load_pull_requests() -> List[PullRequest]:
    records = _load_pr_dicts()
    prs: List[PullRequest] = []
    for record in records:
        prs.append(
            PullRequest(
                number=int(record.get("number") or record.get("id") or 0),
                title=(record.get("title") or "").strip(),
                body=(record.get("body") or "").strip(),
                state=(record.get("state") or "unknown").strip(),
                created_at=_parse_datetime(record.get("created_at")),
                updated_at=_parse_datetime(record.get("updated_at")),
                closed_at=_parse_datetime(record.get("closed_at")),
                merged_at=_parse_datetime(record.get("merged_at")),
            )
        )
    return prs


def refresh_cache() -> None:
    """Clear cached datasets so the next call reloads from disk."""

    load_issues.cache_clear()  # type: ignore[attr-defined]
    load_pull_requests.cache_clear()  # type: ignore[attr-defined]
    load_summary.cache_clear()  # type: ignore[attr-defined]


@lru_cache(maxsize=1)
def load_summary() -> Dict[str, Any]:
    path = EXPORT_DIR / "summary.json"
    try:
        with path.open(encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError:
        return {}
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Failed to parse summary file '{path}': {exc}") from exc
    if not isinstance(data, dict):
        raise RuntimeError("Summary file must contain a JSON object")
    return data
