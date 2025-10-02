# BUSMASTER Modernization Web Interface

The modernization explorer introduces a browser-based façade for the BUSMASTER
project. It turns the exported GitHub issues and pull requests into a searchable
REST API with a responsive dashboard so the team can triage modernization work
without relying on the legacy desktop shell.

## Features

- **FastAPI backend** – Serves the GitHub export with filtering, pagination, and
  summary statistics. Endpoints cover issues, pull requests, labels, and cache
  refresh.
- **Data access layer** – Normalizes the mixed CSV/JSON export formats so the API
  always returns consistent payloads.
- **Responsive frontend** – Vanilla HTML/CSS/JS dashboard that surfaces
  repository stats, label shortcuts, an issue explorer, and pull request lists.
- **One-click refresh** – Reload the export data in-memory without restarting the
  API to pick up new snapshots.

## Project layout

```
WebInterface/
  backend/
    __init__.py
    data_access.py
    main.py
    requirements.txt
  frontend/
    app.js
    index.html
    style.css
```

## Running the prototype locally

1. **Set up a virtual environment and install backend dependencies.**

   ```bash
   cd WebInterface/backend
   python -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

2. **Start the FastAPI server.**

   ```bash
   uvicorn WebInterface.backend.main:app --reload
   ```

   Run the command from the repository root or set `PYTHONPATH` accordingly. The
   server exposes interactive docs at <http://127.0.0.1:8000/docs>.

3. **Serve the frontend assets.** Any static web server works while iterating.

   ```bash
   cd ../frontend
   python -m http.server 8080
   ```

   Visit <http://127.0.0.1:8080>. The JavaScript defaults to the API at
   <http://127.0.0.1:8000>. To point at a different backend, define
   `window.API_BASE` before loading `app.js`.

4. **Refresh data from the UI.** Use the *Reload data* button in the header to
   flush the backend caches and re-read the export files. The dashboard updates
   automatically with the latest counts and filters.

## Available API endpoints

The backend auto-documents itself via the Swagger UI, but the key routes are:

| Endpoint | Description |
| --- | --- |
| `GET /api/stats` | Repository totals, top labels, and export metadata |
| `GET /api/issues` | Paginated issue list with state/label/search filters |
| `GET /api/issues/{number}` | Single issue details |
| `GET /api/issues/labels` | Unique label names for populating filters |
| `GET /api/pull_requests` | Paginated pull request list with search |
| `GET /api/pull_requests/{number}` | Single pull request details |
| `POST /api/cache/refresh` | Clears caches and reloads the export files |

## Next steps

- Mount the dashboard behind the existing BUSMASTER authentication façade once
  available.
- Extend the backend to stream live metrics (build status, nightly tests, etc.).
- Feed the API with generated modernization artifacts (e.g., compatibility
  matrices, driver support tables) to cover more of the desktop UI surface.
