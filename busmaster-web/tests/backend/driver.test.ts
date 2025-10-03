import { describe, it, expect, afterAll } from 'vitest';
import request from 'supertest';
import { app, server } from '../../server/src/index';

describe('Driver API', () => {
  afterAll(() => {
    server.close();
  });

  describe('GET /api/v1/drivers', () => {
    it('should return list of drivers', async () => {
      const response = await request(app).get('/api/v1/drivers');
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('drivers');
      expect(Array.isArray(response.body.drivers)).toBe(true);
    });

    it('should include virtual-can driver', async () => {
      const response = await request(app).get('/api/v1/drivers');
      
      const virtualCan = response.body.drivers.find(
        (d: any) => d.id === 'virtual-can'
      );
      
      expect(virtualCan).toBeDefined();
      expect(virtualCan.available).toBe(true);
    });
  });

  describe('POST /api/v1/drivers/scan', () => {
    it('should scan for hardware', async () => {
      const response = await request(app)
        .post('/api/v1/drivers/scan');
      
      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body).toHaveProperty('devicesFound');
    });
  });

  describe('GET /api/v1/drivers/:driverId/channels', () => {
    it('should return channels for a driver', async () => {
      const response = await request(app)
        .get('/api/v1/drivers/virtual-can/channels');
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('channels');
      expect(Array.isArray(response.body.channels)).toBe(true);
    });
  });
});

