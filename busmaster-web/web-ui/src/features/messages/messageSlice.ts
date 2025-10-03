import { createSlice, PayloadAction } from '@reduxjs/toolkit';

export interface CANMessage {
  id: number;
  data: number[];
  timestamp: number;
  channel: number;
  direction: 'tx' | 'rx';
  extended?: boolean;
}

interface MessageState {
  messages: CANMessage[];
  maxMessages: number;
  paused: boolean;
}

const initialState: MessageState = {
  messages: [],
  maxMessages: 1000,
  paused: false,
};

const messageSlice = createSlice({
  name: 'messages',
  initialState,
  reducers: {
    addMessage: (state, action: PayloadAction<CANMessage>) => {
      state.messages.unshift(action.payload);
      if (state.messages.length > state.maxMessages) {
        state.messages.pop();
      }
    },
    clearMessages: (state) => {
      state.messages = [];
    },
    togglePause: (state) => {
      state.paused = !state.paused;
    },
    setMaxMessages: (state, action: PayloadAction<number>) => {
      state.maxMessages = action.payload;
    },
  },
});

export const { addMessage, clearMessages, togglePause, setMaxMessages } =
  messageSlice.actions;
export default messageSlice.reducer;

