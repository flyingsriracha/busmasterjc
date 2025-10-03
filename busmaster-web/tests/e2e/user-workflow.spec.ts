import { test, expect } from '@playwright/test';

test.describe('BUSMASTER Web User Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000');
  });

  test('should load dashboard page', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  });

  test('should navigate to configuration', async ({ page }) => {
    await page.getByRole('button', { name: 'Configuration' }).click();
    await expect(page.getByRole('heading', { name: 'Configuration' })).toBeVisible();
  });

  test('should connect with virtual CAN', async ({ page }) => {
    // Navigate to configuration
    await page.getByRole('button', { name: 'Configuration' }).click();
    
    // Select virtual CAN
    await page.getByLabel('Hardware Driver').click();
    await page.getByRole('option', { name: 'Virtual CAN' }).click();
    
    // Connect
    await page.getByRole('button', { name: 'Connect' }).click();
    
    // Verify connection indicator turns green
    // (This would check for the green status dot in the header)
    await page.waitForTimeout(1000);
  });

  test('should display messages window', async ({ page }) => {
    await page.getByRole('button', { name: 'Messages' }).click();
    await expect(page.getByRole('heading', { name: 'Message Window' })).toBeVisible();
  });

  test('should navigate to transmit page', async ({ page }) => {
    await page.getByRole('button', { name: 'Transmit' }).click();
    await expect(page.getByRole('heading', { name: 'Transmit Message' })).toBeVisible();
  });

  test('should fill transmit form', async ({ page }) => {
    await page.getByRole('button', { name: 'Transmit' }).click();
    
    // Fill message ID
    await page.getByLabel('Message ID (hex)').fill('0x200');
    
    // Fill data bytes
    await page.getByLabel('Data Bytes (hex)').fill('AA BB CC DD EE FF 00 11');
    
    // Verify send button is enabled
    await expect(page.getByRole('button', { name: 'Send Message' })).toBeEnabled();
  });

  test('complete workflow: connect, view messages, send message', async ({ page }) => {
    // Step 1: Configure and connect
    await page.getByRole('button', { name: 'Configuration' }).click();
    await page.getByLabel('Hardware Driver').click();
    await page.getByRole('option', { name: 'Virtual CAN' }).click();
    await page.getByRole('button', { name: 'Connect' }).click();
    await page.waitForTimeout(500);
    
    // Step 2: View messages
    await page.getByRole('button', { name: 'Messages' }).click();
    await expect(page.getByRole('heading', { name: 'Message Window' })).toBeVisible();
    
    // Step 3: Send a message
    await page.getByRole('button', { name: 'Transmit' }).click();
    await page.getByLabel('Message ID (hex)').fill('0x123');
    await page.getByLabel('Data Bytes (hex)').fill('01 02 03 04');
    await page.getByRole('button', { name: 'Send Message' }).click();
  });
});

