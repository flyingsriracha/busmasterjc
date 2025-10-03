import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import { app, server } from '../../server/src/index';

describe('Connection API', () => {
  afterAll(() => {
    server.close();
  });

  describe('GET /api/v1/connection/status', () => {
    it('should return connection status', async () => {
      const response = await request(app).get('/api/v1/connection/status');
      
      expect(response.status).toBe(200);
      expect(response.body).toHaveProperty('connected');
      expect(response.body).toHaveProperty('driver');
      expect(response.body).toHaveProperty('channels');
      expect(response.body).toHaveProperty('timestamp');
    });

    it('should return disconnected by default', async () => {
      const response = await request(app).get('/api/v1/connection/status');
      
      expect(response.body.connected).toBe(false);
      expect(response.body.driver).toBe(null);
    });
  });

  describe('POST /api/v1/connection/connect', () => {
    it('should require driverId', async () => {
      const response = await request(app)
        .post('/api/v1/connection/connect')
        .send({});
      
      expect(response.status).toBe(400);
      expect(response.body).toHaveProperty('error');
    });

    it('should accept valid connection request', async () => {
      const response = await request(app)
        .post('/api/v1/connection/connect')
        .send({
          driverId: 'virtual-can',
          channels: []
        });
      
      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
    });
  });

  describe('POST /api/v1/connection/disconnect', () => {
    it('should disconnect successfully', async () => {
      const response = await request(app)
        .post('/api/v1/connection/disconnect');
      
      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
    });
  });
});

