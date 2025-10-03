import { Router, Request, Response } from 'express';
import { logger } from '../../utils/logger';

const router = Router();

router.post('/start', async (req: Request, res: Response) => {
  try {
    const { filename, format = 'csv' } = req.body;
    
    logger.info(`Starting logging to: ${filename}`);
    
    res.json({ success: true, message: 'Logging started', filename });
  } catch (error) {
    logger.error('Error starting logging:', error);
    res.status(500).json({ error: 'Failed to start logging' });
  }
});

router.post('/stop', async (req: Request, res: Response) => {
  try {
    logger.info('Stopping logging');
    
    res.json({ success: true, message: 'Logging stopped' });
  } catch (error) {
    logger.error('Error stopping logging:', error);
    res.status(500).json({ error: 'Failed to stop logging' });
  }
});

router.get('/files', async (req: Request, res: Response) => {
  try {
    const files: any[] = [];
    
    res.json({ files });
  } catch (error) {
    logger.error('Error listing log files:', error);
    res.status(500).json({ error: 'Failed to list log files' });
  }
});

export default router;

