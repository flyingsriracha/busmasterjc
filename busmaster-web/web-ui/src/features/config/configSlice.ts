import { createSlice, PayloadAction } from '@reduxjs/toolkit';

interface ConfigState {
  currentConfig: any | null;
  loading: boolean;
  error: string | null;
}

const initialState: ConfigState = {
  currentConfig: null,
  loading: false,
  error: null,
};

const configSlice = createSlice({
  name: 'config',
  initialState,
  reducers: {
    loadConfigRequest: (state) => {
      state.loading = true;
      state.error = null;
    },
    loadConfigSuccess: (state, action: PayloadAction<any>) => {
      state.currentConfig = action.payload;
      state.loading = false;
    },
    loadConfigFailure: (state, action: PayloadAction<string>) => {
      state.loading = false;
      state.error = action.payload;
    },
  },
});

export const { loadConfigRequest, loadConfigSuccess, loadConfigFailure } =
  configSlice.actions;
export default configSlice.reducer;

