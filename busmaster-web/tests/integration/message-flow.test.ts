import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import request from 'supertest';
import { io as ioClient, Socket } from 'socket.io-client';
import { app, server } from '../../server/src/index';

describe('Message Flow Integration', () => {
  let socket: Socket;
  const serverUrl = 'http://localhost:8080';

  beforeAll((done) => {
    socket = ioClient(serverUrl);
    socket.on('connect', done);
  });

  afterAll(() => {
    socket.close();
    server.close();
  });

  it('should receive simulated messages via WebSocket', (done) => {
    socket.on('message:received', (message) => {
      expect(message).toHaveProperty('id');
      expect(message).toHaveProperty('data');
      expect(message).toHaveProperty('timestamp');
      expect(message).toHaveProperty('channel');
      expect(message).toHaveProperty('direction');
      done();
    });

    // Subscribe to messages
    socket.emit('subscribe:messages');
  });

  it('should send message via REST API', async () => {
    const response = await request(app)
      .post('/api/v1/messages/send')
      .send({
        id: 0x100,
        data: [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        extended: false,
        channel: 0
      });

    expect(response.status).toBe(200);
    expect(response.body.success).toBe(true);
  });

  it('should get message buffer via REST API', async () => {
    const response = await request(app)
      .get('/api/v1/messages/buffer?limit=100');

    expect(response.status).toBe(200);
    expect(response.body).toHaveProperty('messages');
    expect(response.body).toHaveProperty('count');
    expect(response.body).toHaveProperty('limit');
  });

  it('should configure filters via REST API', async () => {
    const response = await request(app)
      .post('/api/v1/messages/filter')
      .send({
        filters: [
          { id: 0x100, mask: 0x7FF, type: 'pass' }
        ]
      });

    expect(response.status).toBe(200);
    expect(response.body.success).toBe(true);
  });
});

