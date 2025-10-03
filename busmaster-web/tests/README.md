# BUSMASTER Web - Tests

This directory contains all test suites for BUSMASTER Web.

## Test Structure

```
tests/
├── backend/         # Backend API unit tests
├── frontend/        # Frontend component tests
├── integration/     # Integration tests (API + Frontend)
└── e2e/            # End-to-end tests (full workflow)
```

## Running Tests

### Backend Tests
```bash
cd ../server
npm test
npm run test:coverage
```

### Frontend Tests
```bash
cd ../web-ui
npm test
npm run test:coverage
```

### Integration Tests
```bash
cd integration
npm test
```

### End-to-End Tests
```bash
cd e2e
npm test
```

## Test Coverage Goals

- **Backend**: > 80% code coverage
- **Frontend**: > 70% code coverage
- **Integration**: All critical paths covered
- **E2E**: All user workflows covered

## Writing Tests

### Backend Example (Vitest)
```typescript
import { describe, it, expect } from 'vitest';
import request from 'supertest';
import { app } from '../src/index';

describe('Connection API', () => {
  it('should return connection status', async () => {
    const response = await request(app).get('/api/v1/connection/status');
    expect(response.status).toBe(200);
    expect(response.body).toHaveProperty('connected');
  });
});
```

### Frontend Example (Vitest + React Testing Library)
```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DashboardPage } from '../src/pages/DashboardPage';

describe('DashboardPage', () => {
  it('should render dashboard title', () => {
    render(<DashboardPage />);
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });
});
```

## CI/CD Integration

Tests are automatically run on:
- Every pull request
- Every commit to main branch
- Before deployment

## Test Data

Test fixtures and mock data should be placed in:
- `tests/fixtures/` - Sample data files
- `tests/mocks/` - Mock implementations

