const DEFAULT_API_BASE = `${window.location.protocol}//${window.location.hostname}:8000`;
const API_BASE = window.API_BASE || DEFAULT_API_BASE;

async function fetchJson(path, params, options = {}) {
  const url = new URL(path, API_BASE);
  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== "") {
        url.searchParams.set(key, value);
      }
    });
  }

  const { method = "GET", headers = {}, body, ...rest } = options;
  const fetchOptions = {
    method,
    headers: { Accept: "application/json", ...headers },
    ...rest,
  };

  if (body && typeof body === "object" && !(body instanceof FormData)) {
    fetchOptions.body = JSON.stringify(body);
    fetchOptions.headers["Content-Type"] = "application/json";
  } else if (body) {
    fetchOptions.body = body;
  }

  const response = await fetch(url, fetchOptions);
  if (!response.ok) {
    const detail = await response.text();
    throw new Error(`Request failed: ${response.status} ${detail}`);
  }
  return response.json();
}

function formatDate(value) {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function createLabelPill(text) {
  const pill = document.createElement("span");
  pill.className = "label-pill";
  pill.textContent = text;
  return pill;
}

function applyTemplate(templateId) {
  const template = document.getElementById(templateId);
  return template.content.firstElementChild.cloneNode(true);
}

function updatePager(container, { page, per_page, total }, onChange) {
  const totalPages = Math.max(1, Math.ceil(total / per_page));
  container.innerHTML = "";

  const status = document.createElement("span");
  status.textContent = `Page ${page} of ${totalPages}`;

  const prev = document.createElement("button");
  prev.type = "button";
  prev.textContent = "Previous";
  prev.disabled = page <= 1;
  prev.addEventListener("click", () => {
    Promise.resolve(onChange(page - 1)).catch((error) =>
      console.error("Pagination failed", error),
    );
  });

  const next = document.createElement("button");
  next.type = "button";
  next.textContent = "Next";
  next.disabled = page >= totalPages;
  next.addEventListener("click", () => {
    Promise.resolve(onChange(page + 1)).catch((error) =>
      console.error("Pagination failed", error),
    );
  });

  container.append(prev, status, next);
}

function renderStats(stats) {
  const grid = document.getElementById("stats-grid");
  grid.innerHTML = "";

  const cards = [
    {
      title: "Issues",
      total: stats.issues.total,
      breakdown: `Open ${stats.issues.open} · Closed ${stats.issues.closed}`,
    },
    {
      title: "Pull Requests",
      total: stats.pull_requests.total,
      breakdown: `Open ${stats.pull_requests.open} · Closed ${stats.pull_requests.closed} · Merged ${stats.pull_requests.merged}`,
    },
  ];

  if (stats.repository) {
    cards.unshift({
      title: "Repository",
      total: stats.repository,
      breakdown: stats.export_date ? `Exported ${formatDate(stats.export_date)}` : "",
    });
  }

  cards.forEach((card) => {
    const element = document.createElement("article");
    element.className = "stat-card";

    const heading = document.createElement("h3");
    heading.textContent = card.title;

    const total = document.createElement("strong");
    total.textContent = card.total;

    element.append(heading, total);
    if (card.breakdown) {
      const detail = document.createElement("p");
      detail.textContent = card.breakdown;
      detail.className = "muted";
      element.append(detail);
    }

    grid.append(element);
  });

  const cloud = document.getElementById("label-cloud");
  cloud.innerHTML = "";
  if (!stats.top_labels?.length) {
    cloud.textContent = "No labels found in export.";
    return;
  }

  stats.top_labels.forEach(({ label, count }) => {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.label = label;
    button.textContent = `${label} (${count})`;
    button.addEventListener("click", () => {
      const filters = document.getElementById("issue-filters");
      filters.label.value = label;
      filters.dispatchEvent(new Event("submit"));
      document.querySelectorAll(".label-cloud button").forEach((btn) => btn.classList.remove("active"));
      button.classList.add("active");
    });
    cloud.append(button);
  });
}

function renderIssues(response, repo) {
  const body = document.getElementById("issues-body");
  body.innerHTML = "";

  response.results.forEach((issue) => {
    const row = applyTemplate("issue-row-template");
    row.querySelector(".mono").textContent = `#${issue.number}`;

    const titleCell = row.querySelector(".title");
    const link = document.createElement("a");
    link.href = repo ? `https://github.com/${repo}/issues/${issue.number}` : "#";
    link.target = "_blank";
    link.rel = "noreferrer";
    link.textContent = issue.title || "Untitled";
    titleCell.append(link);

    const labelsCell = row.querySelector(".labels");
    if (issue.labels?.length) {
      issue.labels.forEach((label) => labelsCell.append(createLabelPill(label)));
    } else {
      labelsCell.textContent = "—";
    }

    row.querySelector(".state").textContent = issue.state;
    row.querySelector(".updated").textContent = formatDate(issue.updated_at || issue.created_at);

    body.append(row);
  });

  if (response.results.length === 0) {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    cell.colSpan = 5;
    cell.textContent = "No issues match the current filters.";
    row.append(cell);
    body.append(row);
  }
}

function renderPullRequests(response, repo) {
  const body = document.getElementById("prs-body");
  body.innerHTML = "";

  response.results.forEach((pr) => {
    const row = applyTemplate("pr-row-template");
    row.querySelector(".mono").textContent = `#${pr.number}`;

    const titleCell = row.querySelector(".title");
    const link = document.createElement("a");
    link.href = repo ? `https://github.com/${repo}/pull/${pr.number}` : "#";
    link.target = "_blank";
    link.rel = "noreferrer";
    link.textContent = pr.title || "Untitled";
    titleCell.append(link);

    row.querySelector(".state").textContent = pr.state;
    row.querySelector(".updated").textContent = formatDate(pr.updated_at || pr.created_at);

    body.append(row);
  });

  if (response.results.length === 0) {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    cell.colSpan = 4;
    cell.textContent = "No pull requests match the current filters.";
    row.append(cell);
    body.append(row);
  }
}

async function populateLabelFilter() {
  const select = document.querySelector("#issue-filters select[name='label']");
  try {
    const { labels } = await fetchJson("/api/issues/labels");
    Array.from(select.querySelectorAll("option:not([value=''])")).forEach((option) =>
      option.remove(),
    );
    labels.forEach((label) => {
      const option = document.createElement("option");
      option.value = label;
      option.textContent = label;
      select.append(option);
    });
  } catch (error) {
    console.error("Failed to load labels", error);
  }
}

async function main() {
  let stats = await fetchJson("/api/stats");
  renderStats(stats);
  await populateLabelFilter();

  const issueFilters = document.getElementById("issue-filters");
  const issuePager = document.getElementById("issue-pager");

  async function loadIssues(page = 1) {
    const formData = new FormData(issueFilters);
    const params = Object.fromEntries(formData.entries());
    params.page = page;
    params.per_page = 20;
    const response = await fetchJson("/api/issues", params);
    renderIssues(response, stats.repository);
    updatePager(issuePager, response, loadIssues);
  }

  issueFilters.addEventListener("submit", (event) => {
    event.preventDefault();
    loadIssues(1).catch((error) => console.error("Failed to load issues", error));
  });

  const prFilters = document.getElementById("pr-filters");
  const prPager = document.getElementById("pr-pager");

  async function loadPullRequests(page = 1) {
    const formData = new FormData(prFilters);
    const params = Object.fromEntries(formData.entries());
    params.page = page;
    params.per_page = 20;
    const response = await fetchJson("/api/pull_requests", params);
    renderPullRequests(response, stats.repository);
    updatePager(prPager, response, loadPullRequests);
  }

  prFilters.addEventListener("submit", (event) => {
    event.preventDefault();
    loadPullRequests(1).catch((error) => console.error("Failed to load pull requests", error));
  });

  const reloadButton = document.getElementById("reload-cache");
  const reloadStatus = document.getElementById("reload-status");
  reloadButton.addEventListener("click", async () => {
    reloadButton.disabled = true;
    reloadStatus.textContent = "Refreshing data…";
    try {
      await fetchJson("/api/cache/refresh", null, { method: "POST" });
      stats = await fetchJson("/api/stats");
      renderStats(stats);
      await populateLabelFilter();
      await Promise.all([loadIssues(1), loadPullRequests(1)]);
      reloadStatus.textContent = "Data refreshed";
      setTimeout(() => {
        reloadStatus.textContent = "";
      }, 2500);
    } catch (error) {
      reloadStatus.textContent = "Failed to refresh";
      console.error(error);
    } finally {
      reloadButton.disabled = false;
    }
  });

  // Initial load
  await loadIssues();
  await loadPullRequests();
}

main().catch((error) => {
  console.error(error);
  const mainEl = document.querySelector("main");
  const message = document.createElement("p");
  message.textContent = "Failed to load data from the modernization API. Check the console for details.";
  mainEl.prepend(message);
});
