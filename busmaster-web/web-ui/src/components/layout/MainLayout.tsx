import { ReactNode } from 'react';
import { Box, AppBar, Toolbar, Typography, Container, Button } from '@mui/material';
import { Link as RouterLink } from 'react-router-dom';
import { useSelector } from 'react-redux';
import { RootState } from '@/store';

interface MainLayoutProps {
  children: ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  const { connected } = useSelector((state: RootState) => state.connection);

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <AppBar position="static">
        <Toolbar>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            BUSMASTER Web
          </Typography>
          <Button color="inherit" component={RouterLink} to="/">
            Dashboard
          </Button>
          <Button color="inherit" component={RouterLink} to="/messages">
            Messages
          </Button>
          <Button color="inherit" component={RouterLink} to="/transmit">
            Transmit
          </Button>
          <Button color="inherit" component={RouterLink} to="/configuration">
            Configuration
          </Button>
          <Box
            sx={{
              ml: 2,
              width: 12,
              height: 12,
              borderRadius: '50%',
              backgroundColor: connected ? 'success.main' : 'error.main',
            }}
          />
        </Toolbar>
      </AppBar>
      <Container maxWidth="xl" sx={{ mt: 4, mb: 4, flexGrow: 1 }}>
        {children}
      </Container>
    </Box>
  );
}

