import '@testing-library/jest-dom';
import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

// Cleanup after each test case (e.g., clearing jsdom)
afterEach(() => {
  cleanup();
});

// Mock environment variables
process.env.NODE_ENV = 'test';
process.env.PORT = '8080';
process.env.HOST = 'localhost';

