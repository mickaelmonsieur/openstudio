import { createContext, useContext } from 'react';

export const StationContext = createContext({ stationId: '', stations: [] });

export function useStation() {
  return useContext(StationContext);
}
