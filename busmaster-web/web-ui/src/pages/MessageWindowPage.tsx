import { Box, Typography, Paper, IconButton, Tooltip } from '@mui/material';
import { Pause, PlayArrow, Clear } from '@mui/icons-material';
import { useSelector, useDispatch } from 'react-redux';
import { RootState } from '@/store';
import { togglePause, clearMessages } from '@/features/messages/messageSlice';

export function MessageWindowPage() {
  const dispatch = useDispatch();
  const { messages, paused } = useSelector((state: RootState) => state.messages);

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
        <Typography variant="h4">
          Message Window
        </Typography>
        <Box>
          <Tooltip title={paused ? 'Resume' : 'Pause'}>
            <IconButton onClick={() => dispatch(togglePause())} color="primary">
              {paused ? <PlayArrow /> : <Pause />}
            </IconButton>
          </Tooltip>
          <Tooltip title="Clear">
            <IconButton onClick={() => dispatch(clearMessages())} color="primary">
              <Clear />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>
      <Paper sx={{ p: 2, height: 'calc(100vh - 200px)', overflow: 'auto' }}>
        {messages.length === 0 ? (
          <Typography color="text.secondary" textAlign="center" sx={{ mt: 4 }}>
            No messages to display. Connect to hardware to start receiving messages.
          </Typography>
        ) : (
          <Box sx={{ fontFamily: 'monospace', fontSize: '0.875rem' }}>
            {messages.map((msg, index) => (
              <Box key={index} sx={{ mb: 0.5 }}>
                <span style={{ marginRight: '12px' }}>
                  {new Date(msg.timestamp).toLocaleTimeString()}
                </span>
                <span style={{ marginRight: '12px', color: msg.direction === 'tx' ? '#4caf50' : '#2196f3' }}>
                  {msg.direction.toUpperCase()}
                </span>
                <span style={{ marginRight: '12px' }}>
                  0x{msg.id.toString(16).toUpperCase().padStart(3, '0')}
                </span>
                <span>
                  [{msg.data.map(b => b.toString(16).toUpperCase().padStart(2, '0')).join(' ')}]
                </span>
              </Box>
            ))}
          </Box>
        )}
      </Paper>
    </Box>
  );
}

