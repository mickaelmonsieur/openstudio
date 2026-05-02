const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('openStudioLauncher', {
  getState: () => ipcRenderer.invoke('launcher:get-state'),
  saveConfig: (config) => ipcRenderer.invoke('launcher:save-config', config),
  openAdmin: () => ipcRenderer.invoke('launcher:open-admin'),
  minimize: () => ipcRenderer.invoke('launcher:minimize'),
  hide: () => ipcRenderer.invoke('launcher:hide')
});
