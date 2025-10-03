import { Express, Router } from 'express';
import swaggerJsdoc from 'swagger-jsdoc';
import swaggerUi from 'swagger-ui-express';
import connectionRoutes from './routes/connection';
import driverRoutes from './routes/driver';
import messageRoutes from './routes/message';
import databaseRoutes from './routes/database';
import configRoutes from './routes/config';
import loggingRoutes from './routes/logging';
import statisticsRoutes from './routes/statistics';

const swaggerOptions = {
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'BUSMASTER Web API',
      version: '0.1.0',
      description: 'REST API for BUSMASTER Web - CAN/LIN Bus Analysis Tool',
      license: {
        name: 'LGPL-3.0',
        url: 'https://www.gnu.org/licenses/lgpl-3.0.html',
      },
    },
    servers: [
      {
        url: 'http://localhost:8080',
        description: 'Development server',
      },
    ],
    tags: [
      { name: 'Connection', description: 'Hardware connection management' },
      { name: 'Driver', description: 'Hardware driver operations' },
      { name: 'Message', description: 'CAN/LIN message operations' },
      { name: 'Database', description: 'Database file management' },
      { name: 'Configuration', description: 'Configuration management' },
      { name: 'Logging', description: 'Message logging operations' },
      { name: 'Statistics', description: 'Network statistics and monitoring' },
    ],
  },
  apis: ['./src/api/routes/*.ts'], // Path to API docs
};

const swaggerSpec = swaggerJsdoc(swaggerOptions);

export function setupRoutes(app: Express): void {
  // API version prefix
  const apiRouter = Router();

  // Mount route modules
  apiRouter.use('/connection', connectionRoutes);
  apiRouter.use('/drivers', driverRoutes);
  apiRouter.use('/messages', messageRoutes);
  apiRouter.use('/database', databaseRoutes);
  apiRouter.use('/config', configRoutes);
  apiRouter.use('/logging', loggingRoutes);
  apiRouter.use('/statistics', statisticsRoutes);

  // Mount API router
  app.use('/api/v1', apiRouter);

  // Swagger documentation
  app.use('/api-docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec, {
    explorer: true,
    customSiteTitle: 'BUSMASTER Web API Documentation',
  }));

  // Swagger JSON
  app.get('/api-docs.json', (req, res) => {
    res.setHeader('Content-Type', 'application/json');
    res.send(swaggerSpec);
  });
}

