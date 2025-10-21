import React from 'react';

export interface DockContextValue {
  value: string;
  onValueChange: (value: string) => void;
}

export const DockContext = React.createContext<DockContextValue | null>(null);

export const useDockContext = () => {
  const context = React.useContext(DockContext);
  if (!context) {
    throw new Error('Dock compound components must be wrapped in <Dock />');
  }
  return context;
};
