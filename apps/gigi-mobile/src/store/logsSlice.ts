import { createSlice, type PayloadAction } from '@reduxjs/toolkit';

export interface LogEntry {
  id: string;
  timestamp: string; // Store as string to avoid serialization issues with Date objects
  event: string;
  data: any;
  type: 'info' | 'success' | 'warning' | 'error';
}

interface LogsState {
  logs: LogEntry[];
}

const initialState: LogsState = {
  logs: [],
};

const logsSlice = createSlice({
  name: 'logs',
  initialState,
  reducers: {
    addLog: (state, action: PayloadAction<{
      event: string;
      data: any;
      type?: LogEntry['type'];
    }>) => {
      const { event, data, type = 'info' } = action.payload;
      const newLog: LogEntry = {
        id: `${Date.now()}-${Math.random()}`,
        timestamp: new Date().toISOString(),
        event,
        data,
        type,
      };
      
      // Add to beginning and keep only last 50 logs
      state.logs = [newLog, ...state.logs.slice(0, 49)];
    },
    clearLogs: (state) => {
      state.logs = [];
    },
  },
});

export const { addLog, clearLogs } = logsSlice.actions;
export default logsSlice.reducer;