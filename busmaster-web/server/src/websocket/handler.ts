import { Server as SocketIOServer, Socket } from 'socket.io';
import { logger } from '../utils/logger';

export function setupWebSocket(io: SocketIOServer): void {
  io.on('connection', (socket: Socket) => {
    logger.info(`WebSocket client connected: ${socket.id}`);

    // Send welcome message
    socket.emit('connected', {
      message: 'Connected to BUSMASTER Web',
      timestamp: new Date().toISOString(),
    });

    // Handle message subscription
    socket.on('subscribe:messages', () => {
      logger.info(`Client ${socket.id} subscribed to messages`);
      socket.join('messages');
    });

    socket.on('unsubscribe:messages', () => {
      logger.info(`Client ${socket.id} unsubscribed from messages`);
      socket.leave('messages');
    });

    // Handle statistics subscription
    socket.on('subscribe:statistics', () => {
      logger.info(`Client ${socket.id} subscribed to statistics`);
      socket.join('statistics');
    });

    socket.on('unsubscribe:statistics', () => {
      logger.info(`Client ${socket.id} unsubscribed from statistics`);
      socket.leave('statistics');
    });

    // Handle disconnection
    socket.on('disconnect', (reason) => {
      logger.info(`WebSocket client disconnected: ${socket.id}, reason: ${reason}`);
    });

    // Handle errors
    socket.on('error', (error) => {
      logger.error(`WebSocket error for client ${socket.id}:`, error);
    });
  });

  // Example: Simulate sending CAN messages (for testing)
  // In production, this would be triggered by actual hardware events
  if (process.env.NODE_ENV === 'development') {
    setInterval(() => {
      // Simulate a CAN message
      const message = {
        id: Math.floor(Math.random() * 0x7FF),
        data: Array.from({ length: 8 }, () => Math.floor(Math.random() * 256)),
        timestamp: Date.now(),
        channel: 0,
        direction: 'rx',
      };

      // Broadcast to all clients subscribed to messages
      io.to('messages').emit('message:received', message);
    }, 1000); // Send message every second
  }

  logger.info('WebSocket handler initialized');
}

// Helper function to broadcast message to all connected clients
export function broadcastMessage(io: SocketIOServer, event: string, data: any): void {
  io.emit(event, data);
}

// Helper function to broadcast to a specific room
export function broadcastToRoom(
  io: SocketIOServer,
  room: string,
  event: string,
  data: any
): void {
  io.to(room).emit(event, data);
}

