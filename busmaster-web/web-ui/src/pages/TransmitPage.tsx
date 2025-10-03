import { useState } from 'react';
import { Box, Typography, Paper, TextField, Button, Grid } from '@mui/material';

export function TransmitPage() {
  const [messageId, setMessageId] = useState('0x100');
  const [dataBytes, setDataBytes] = useState('00 00 00 00 00 00 00 00');

  const handleSend = () => {
    console.log('Sending message:', { messageId, dataBytes });
    // TODO: Call API to send message
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Transmit Message
      </Typography>
      <Paper sx={{ p: 3 }}>
        <Grid container spacing={3}>
          <Grid item xs={12} md={6}>
            <TextField
              fullWidth
              label="Message ID (hex)"
              value={messageId}
              onChange={(e) => setMessageId(e.target.value)}
              helperText="Enter message ID in hexadecimal format (e.g., 0x100)"
            />
          </Grid>
          <Grid item xs={12} md={6}>
            <TextField
              fullWidth
              label="Data Bytes (hex)"
              value={dataBytes}
              onChange={(e) => setDataBytes(e.target.value)}
              helperText="Enter data bytes separated by spaces (e.g., 01 02 03 04)"
            />
          </Grid>
          <Grid item xs={12}>
            <Button variant="contained" color="primary" onClick={handleSend}>
              Send Message
            </Button>
          </Grid>
        </Grid>
      </Paper>
    </Box>
  );
}

