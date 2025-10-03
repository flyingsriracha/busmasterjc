import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Provider } from 'react-redux';
import { configureStore } from '@reduxjs/toolkit';
import { DashboardPage } from '../../web-ui/src/pages/DashboardPage';
import connectionReducer from '../../web-ui/src/features/connection/connectionSlice';
import messageReducer from '../../web-ui/src/features/messages/messageSlice';
import configReducer from '../../web-ui/src/features/config/configSlice';

const createMockStore = () => {
  return configureStore({
    reducer: {
      connection: connectionReducer,
      messages: messageReducer,
      config: configReducer,
    },
  });
};

describe('DashboardPage', () => {
  it('should render dashboard title', () => {
    const store = createMockStore();
    
    render(
      <Provider store={store}>
        <DashboardPage />
      </Provider>
    );
    
    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('should display connection status', () => {
    const store = createMockStore();
    
    render(
      <Provider store={store}>
        <DashboardPage />
      </Provider>
    );
    
    expect(screen.getByText('Connection Status')).toBeInTheDocument();
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
  });

  it('should display message statistics', () => {
    const store = createMockStore();
    
    render(
      <Provider store={store}>
        <DashboardPage />
      </Provider>
    );
    
    expect(screen.getByText('Messages')).toBeInTheDocument();
    expect(screen.getByText('Total Messages:')).toBeInTheDocument();
  });

  it('should display quick start guide', () => {
    const store = createMockStore();
    
    render(
      <Provider store={store}>
        <DashboardPage />
      </Provider>
    );
    
    expect(screen.getByText('Quick Start')).toBeInTheDocument();
  });
});

