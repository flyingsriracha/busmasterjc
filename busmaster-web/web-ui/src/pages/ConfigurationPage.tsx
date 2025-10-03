import { Box, Typography, Paper, FormControl, InputLabel, Select, MenuItem, Button } from '@mui/material';
import { useState } from 'react';
import { useDispatch } from 'react-redux';
import { connectRequest, connectSuccess } from '@/features/connection/connectionSlice';

export function ConfigurationPage() {
  const dispatch = useDispatch();
  const [selectedDriver, setSelectedDriver] = useState('virtual-can');

  const handleConnect = () => {
    dispatch(connectRequest());
    // TODO: Call API to connect
    setTimeout(() => {
      dispatch(connectSuccess({ driver: selectedDriver, channels: [] }));
    }, 1000);
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Configuration
      </Typography>
      <Paper sx={{ p: 3 }}>
        <FormControl fullWidth sx={{ mb: 3 }}>
          <InputLabel>Hardware Driver</InputLabel>
          <Select
            value={selectedDriver}
            onChange={(e) => setSelectedDriver(e.target.value)}
            label="Hardware Driver"
          >
            <MenuItem value="virtual-can">Virtual CAN (Simulator)</MenuItem>
            <MenuItem value="peak-usb" disabled>PEAK USB</MenuItem>
            <MenuItem value="vector-xl" disabled>Vector XL</MenuItem>
            <MenuItem value="etas-boa" disabled>ETAS BOA</MenuItem>
          </Select>
        </FormControl>
        <Button variant="contained" color="primary" onClick={handleConnect}>
          Connect
        </Button>
      </Paper>
    </Box>
  );
}

