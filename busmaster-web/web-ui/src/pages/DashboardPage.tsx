import { Grid, Paper, Typography, Box } from '@mui/material';
import { useSelector } from 'react-redux';
import { RootState } from '@/store';

export function DashboardPage() {
  const { connected, driver } = useSelector((state: RootState) => state.connection);
  const { messages } = useSelector((state: RootState) => state.messages);

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>
      <Grid container spacing={3}>
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Connection Status
            </Typography>
            <Typography>
              Status: <strong>{connected ? 'Connected' : 'Disconnected'}</strong>
            </Typography>
            <Typography>
              Driver: <strong>{driver || 'None'}</strong>
            </Typography>
          </Paper>
        </Grid>
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Messages
            </Typography>
            <Typography>
              Total Messages: <strong>{messages.length}</strong>
            </Typography>
            <Typography>
              Messages/sec: <strong>0</strong>
            </Typography>
          </Paper>
        </Grid>
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Bus Load
            </Typography>
            <Typography>
              Current: <strong>0%</strong>
            </Typography>
            <Typography>
              Peak: <strong>0%</strong>
            </Typography>
          </Paper>
        </Grid>
        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Quick Start
            </Typography>
            <Typography paragraph>
              Welcome to BUSMASTER Web - A modern, web-based CAN/LIN bus analysis tool
            </Typography>
            <Typography variant="body2" color="text.secondary">
              1. Go to Configuration to select and configure hardware<br />
              2. Click Connect to start monitoring<br />
              3. View real-time messages in the Messages window<br />
              4. Send messages from the Transmit window
            </Typography>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
}

