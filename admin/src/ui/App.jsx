import { useEffect, useMemo, useState } from 'react';
import { StationContext } from './StationContext.jsx';
import { CrudPage } from './crud/CrudPage.jsx';
import { CategoryPage } from './pages/CategoryPage.jsx';
import { artistsResource } from './resources/artists.js';
import { stationsResource } from './resources/stations.js';
import { usersResource } from './resources/users.js';
import { StationsPage } from './pages/StationsPage.jsx';
import { SchedulesPage } from './pages/SchedulesPage.jsx';
import { TemplatesPage } from './pages/TemplatesPage.jsx';
import { TracksPage } from './pages/TracksPage.jsx';
import { CuePage } from './pages/CuePage.jsx';
import { LogPage } from './pages/LogPage.jsx';
import { playLogResource } from './resources/play-log.js';
import { automixLogResource } from './resources/automix-log.js';

const moduleGroups = [
  {
    title: 'General',
    items: [
      { label: 'Stations', path: '/stations' }
    ]
  },
  {
    title: 'Media',
    items: [
      { label: 'Category', path: '/category' },
      { label: 'Artists', path: '/artists' },
      { label: 'Tracks', path: '/tracks' }
    ]
  },
  {
    title: 'Automation',
    items: [
      { label: 'Templates', path: '/templates' },
      { label: 'Schedules', path: '/schedules' },
      { label: 'Events', path: '/events' }
    ]
  },
  {
    title: 'Advertising',
    items: [
      { label: 'Advertisers', path: '/advertisers' },
      { label: 'Contacts', path: '/contacts' },
      { label: 'Campaigns', path: '/campaigns' },
      { label: 'Campaigns Tracks', path: '/campaigns-tracks' }
    ]
  },
  {
    title: 'Admin',
    items: [
      { label: 'Database', path: '/database' },
      { label: 'Users', path: '/users' },
      { label: 'Play Log', path: '/play-log' },
      { label: 'Auto Mix Log', path: '/auto-mix-log' }
    ]
  }
];

const modules = moduleGroups.flatMap((group) =>
  group.items.map((item) => ({ ...item, group: group.title }))
);

export function App() {
  const [apiStatus, setApiStatus] = useState('Checking server...');
  const [databaseConfig, setDatabaseConfig] = useState({
    database: 'openstudio',
    host: 'localhost',
    password: 'openstudio',
    port: 5432,
    user: 'openstudio'
  });
  const [databaseStatus, setDatabaseStatus] = useState({
    connected: false,
    error: null,
    checkedAt: null
  });
  const [databaseSaving, setDatabaseSaving] = useState(false);
  const [currentPath, setCurrentPath] = useState(window.location.pathname);
  const [stations, setStations] = useState([]);
  const [stationId, setStationId] = useState('');

  useEffect(() => {
    fetch('/api/hello')
      .then((response) => response.json())
      .then((payload) => setApiStatus(`${payload.app}: ${payload.status}`))
      .catch((error) => setApiStatus(`API unavailable: ${error.message}`));
  }, []);

  useEffect(() => {
    fetch('/api/database')
      .then((response) => response.json())
      .then((payload) => {
        setDatabaseConfig(payload.config);
        setDatabaseStatus(payload.status);
      })
      .catch((error) => {
        setDatabaseStatus({
          connected: false,
          error: error.message,
          checkedAt: new Date().toISOString()
        });
      });
  }, []);

  useEffect(() => {
    const onPopState = () => setCurrentPath(window.location.pathname);
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, []);

  useEffect(() => {
    if (!databaseStatus.connected) return;
    fetch('/api/stations')
      .then((r) => r.json())
      .then((payload) => {
        const rows = payload.rows || [];
        setStations(rows);
        setStationId((prev) => prev || (rows[0] ? String(rows[0].id) : ''));
      })
      .catch(() => {});
  }, [databaseStatus.connected]);

  const activeModule = useMemo(() => {
    if (currentPath.startsWith('/tracks/cue/')) {
      return modules.find((module) => module.path === '/tracks') || modules[0];
    }

    return modules.find((module) => module.path === currentPath) || modules[0];
  }, [currentPath]);

  function navigate(event, path) {
    event.preventDefault();
    window.history.pushState({}, '', path);
    setCurrentPath(path);
  }

  function updateDatabaseField(field, value) {
    setDatabaseConfig((current) => ({
      ...current,
      [field]: field === 'port' ? Number(value) : value
    }));
  }

  async function saveDatabaseConfig(event) {
    event.preventDefault();
    setDatabaseSaving(true);
    try {
      const response = await fetch('/api/database', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(databaseConfig)
      });
      const payload = await response.json();
      setDatabaseConfig(payload.config);
      setDatabaseStatus(payload.status);
    } catch (error) {
      setDatabaseStatus({
        connected: false,
        error: error.message,
        checkedAt: new Date().toISOString()
      });
    } finally {
      setDatabaseSaving(false);
    }
  }

  function renderContent() {
    if (activeModule.path === '/stations') {
      return (
        <section className="content-panel">
          <StationsPage />
        </section>
      );
    }

    if (activeModule.path === '/artists') {
      return (
        <section className="content-panel">
          <CrudPage resource={artistsResource} />
        </section>
      );
    }

    if (activeModule.path === '/templates') {
      return (
        <section className="content-panel wide-panel">
          <TemplatesPage />
        </section>
      );
    }

    if (activeModule.path === '/schedules') {
      return (
        <section className="content-panel">
          <SchedulesPage />
        </section>
      );
    }

    if (activeModule.path === '/category') {
      return (
        <section className="content-panel wide-panel">
          <CategoryPage />
        </section>
      );
    }

    if (activeModule.path === '/users') {
      return (
        <section className="content-panel">
          <CrudPage resource={usersResource} />
        </section>
      );
    }

    if (activeModule.path === '/tracks') {
      const cueMatch = currentPath.match(/^\/tracks\/cue\/(\d+)$/);
      if (cueMatch) {
        return (
          <section className="content-panel wide-panel">
            <CuePage trackId={cueMatch[1]} />
          </section>
        );
      }

      return (
        <section className="content-panel">
          <TracksPage />
        </section>
      );
    }

    if (activeModule.path === '/play-log') {
      return (
        <section className="content-panel">
          <LogPage resource={playLogResource} />
        </section>
      );
    }

    if (activeModule.path === '/auto-mix-log') {
      return (
        <section className="content-panel">
          <LogPage resource={automixLogResource} />
        </section>
      );
    }

    if (activeModule.path === '/database') {
      return (
        <section className="content-panel">
          <h2>Connexion Database</h2>
          <form className="database-form" onSubmit={saveDatabaseConfig}>
            <label>
              <span>Host</span>
              <input
                value={databaseConfig.host}
                onChange={(event) => updateDatabaseField('host', event.target.value)}
              />
            </label>
            <label>
              <span>Port</span>
              <input
                min="1"
                max="65535"
                type="number"
                value={databaseConfig.port}
                onChange={(event) => updateDatabaseField('port', event.target.value)}
              />
            </label>
            <label>
              <span>Database</span>
              <input
                value={databaseConfig.database}
                onChange={(event) => updateDatabaseField('database', event.target.value)}
              />
            </label>
            <label>
              <span>User</span>
              <input
                value={databaseConfig.user}
                onChange={(event) => updateDatabaseField('user', event.target.value)}
              />
            </label>
            <label>
              <span>Password</span>
              <input
                type="password"
                value={databaseConfig.password}
                onChange={(event) => updateDatabaseField('password', event.target.value)}
              />
            </label>
            <div className="form-actions">
              <button className="primary-button" disabled={databaseSaving} type="submit">
                {databaseSaving ? 'Saving...' : 'Save'}
              </button>
            </div>
          </form>
        </section>
      );
    }

    return (
      <section className="content-panel">
        <p className="panel-kicker">Hello World</p>
        <h2>{activeModule.label}</h2>
        <p>
          This route is ready for the future {activeModule.label} module. Data
          tables, filters, editors, and actions will live here.
        </p>
      </section>
    );
  }

  return (
    <StationContext value={{ stationId, stations }}>
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">OS</div>
          <div>
            <div className="brand-name">OpenStudio</div>
            <div className="brand-subtitle">Admin</div>
          </div>
        </div>

        {stations.length > 0 ? (
          <div className="station-picker">
            <label htmlFor="sidebar-station">Station</label>
            <select
              id="sidebar-station"
              value={stationId}
              onChange={(e) => setStationId(e.target.value)}
            >
              {stations.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </div>
        ) : null}

        <nav className="navigation" aria-label="Admin modules">
          {moduleGroups.map((group) => (
            <section className="nav-group" key={group.title}>
              <h2>{group.title}</h2>
              {group.items.map((item) => {
                const active = item.path === activeModule.path;
                return (
                  <a
                    className={active ? 'nav-link active' : 'nav-link'}
                    href={item.path}
                    key={item.path}
                    onClick={(event) => navigate(event, item.path)}
                  >
                    {item.label}
                  </a>
                );
              })}
            </section>
          ))}
        </nav>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeModule.group}</p>
            <h1>{activeModule.label}</h1>
          </div>
          <div className="server-pill">{apiStatus}</div>
        </header>

        {renderContent()}
      </section>

      <footer className="status-bar">
        <span className="db-status">
          DB:{' '}
          <strong className={databaseStatus.connected ? 'ok' : 'error'}>
            {databaseStatus.connected ? 'Connected' : 'Disconnected'}
          </strong>
        </span>
      </footer>
    </main>
    </StationContext>
  );
}
