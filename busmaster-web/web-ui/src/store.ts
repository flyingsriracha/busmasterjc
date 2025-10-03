import { configureStore } from '@reduxjs/toolkit';
import connectionReducer from './features/connection/connectionSlice';
import messageReducer from './features/messages/messageSlice';
import configReducer from './features/config/configSlice';

export const store = configureStore({
  reducer: {
    connection: connectionReducer,
    messages: messageReducer,
    config: configReducer,
  },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

