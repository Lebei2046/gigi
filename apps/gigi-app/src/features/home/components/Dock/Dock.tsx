import React, { useState } from 'react';
import { DockContext, useDockContext, type DockContextValue } from './DockContext';

interface DockProps {
  defaultValue?: string;
  value?: string;
  onValueChange?: (value: string) => void;
  className?: string;
  children: React.ReactNode;
}

const Dock: React.FC<DockProps> & {
  List: React.FC<{ className?: string; children: React.ReactNode }>;
  Trigger: React.FC<{ value: string; className?: string; children: React.ReactNode }>;
  Content: React.FC<{ value: string; className?: string; children: React.ReactNode }>;
} = ({
  defaultValue = '',
  value: valueProp,
  onValueChange: onValueChangeProp,
  className = '',
  children,
}) => {
    const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue);
    const isControlled = valueProp !== undefined;
    const value = isControlled ? valueProp : uncontrolledValue;

    const onValueChange = (newValue: string) => {
      if (!isControlled) {
        setUncontrolledValue(newValue);
      }
      onValueChangeProp?.(newValue);
    };

    const contextValue: DockContextValue = {
      value,
      onValueChange,
    };

    return (
      <DockContext.Provider value={contextValue}>
        <div className={className}>
          {children}
        </div>
      </DockContext.Provider>
    );
  };

// Define composite components
Dock.List = function DockList({ className = '', children }: { className?: string; children: React.ReactNode }) {
  return (
    <div
      className={`dock ${className}`}
      role="tablist"
    >
      {children}
    </div>
  );
};

Dock.Trigger = function DockTrigger({ value, className = '', children }: { value: string; className?: string; children: React.ReactNode }) {
  const { value: currentValue, onValueChange } = useDockContext();
  const isActive = currentValue === value;

  return (
    <button
      className={`dock-item ${isActive ? 'dock-active' : ''} ${className}`}
      role="tab"
      aria-selected={isActive}
      aria-controls={`dock-content-${value}`}
      id={`dock-trigger-${value}`}
      onClick={() => onValueChange(value)}
      tabIndex={isActive ? 0 : -1}
    >
      {children}
    </button>
  );
};

Dock.Content = function DockContent({ value, className = '', children }: { value: string; className?: string; children: React.ReactNode }) {
  const { value: currentValue } = useDockContext();
  const isActive = currentValue === value;

  return (
    <div
      className={`${isActive ? 'block' : 'hidden'} ${className}`}
      role="tabpanel"
      aria-labelledby={`dock-trigger-${value}`}
      id={`dock-content-${value}`}
      hidden={!isActive}
    >
      {children}
    </div>
  );
};

export default Dock;
