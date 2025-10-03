import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface ConnectionState {
  connected: boolean;
  driver: string | null;
  channels: any[];
  loading: boolean;
  error: string | null;
}

const initialState: ConnectionState = {
  connected: false,
  driver: null,
  channels: [],
  loading: false,
  error: null,
};

const connectionSlice = createSlice({
  name: 'connection',
  initialState,
  reducers: {
    connectRequest: (state) => {
      state.loading = true;
      state.error = null;
    },
    connectSuccess: (state, action: PayloadAction<{ driver: string; channels: any[] }>) => {
      state.connected = true;
      state.driver = action.payload.driver;
      state.channels = action.payload.channels;
      state.loading = false;
    },
    connectFailure: (state, action: PayloadAction<string>) => {
      state.loading = false;
      state.error = action.payload;
    },
    disconnect: (state) => {
      state.connected = false;
      state.driver = null;
      state.channels = [];
    },
  },
});

export const { connectRequest, connectSuccess, connectFailure, disconnect } =
  connectionSlice.actions;
export default connectionSlice.reducer;

