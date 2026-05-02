# OpenStudio Admin

Node/Electron launcher for the future OpenStudio administration app.

Current scaffold:

- Electron launcher with tray/menu.
- Local settings window for bind address, ports, autostart, and start minimized.
- Node/Express web server on port `7061`.
- Node/Express control/status server on port `7063`.
- React/Vite Hello World page.

Run in development:

```bash
cd admin
npm install
npm run dev
```

Build the web UI before packaging:

```bash
npm run build:web
```
