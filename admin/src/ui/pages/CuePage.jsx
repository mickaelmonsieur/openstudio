import { useEffect, useRef, useState } from 'react';

const CUE_MARKERS = [
  { field: 'cue_in', label: 'CUE IN', color: '#1e88e5' },
  { field: 'intro', label: 'INTRO', color: '#43a047' },
  { field: 'hook_in', label: 'HOOK IN', color: '#f9a825' },
  { field: 'hook_out', label: 'HOOK OUT', color: '#8e24aa' },
  { field: 'loop_in', label: 'LOOP IN', color: '#00acc1' },
  { field: 'loop_out', label: 'LOOP OUT', color: '#fb8c00' },
  { field: 'outro', label: 'OUTRO', color: '#7cb342' },
  { field: 'cue_out', label: 'CUE OUT', color: '#6d4c41' }
];

export function CuePage({ trackId }) {
  const canvasRef = useRef(null);
  const audioRef = useRef(null);
  const waveformRef = useRef(null);
  const animationRef = useRef(null);
  const [waveform, setWaveform] = useState(null);
  const [playing, setPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [savingMarker, setSavingMarker] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    let cancelled = false;

    async function loadWaveform() {
      setLoading(true);
      setError(null);
      try {
        const response = await fetch(`/api/tracks/${trackId}/waveform?points=2400`);
        const payload = await response.json().catch(() => ({}));
        if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);
        if (!cancelled) {
          waveformRef.current = payload;
          setWaveform(payload);
        }
      } catch (err) {
        if (!cancelled) setError(err.message);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    loadWaveform();

    return () => {
      cancelled = true;
    };
  }, [trackId]);

  useEffect(() => {
    if (!waveform) return undefined;

    waveformRef.current = waveform;
    const draw = () => drawCueCanvas(canvasRef.current, waveform, audioRef.current);
    draw();
    window.addEventListener('resize', draw);
    return () => window.removeEventListener('resize', draw);
  }, [waveform]);

  useEffect(() => {
    if (!playing) {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
        animationRef.current = null;
      }
      drawCueCanvas(canvasRef.current, waveformRef.current, audioRef.current);
      return undefined;
    }

    function tick() {
      drawCueCanvas(canvasRef.current, waveformRef.current, audioRef.current);
      animationRef.current = requestAnimationFrame(tick);
    }

    animationRef.current = requestAnimationFrame(tick);
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
        animationRef.current = null;
      }
    };
  }, [playing]);

  return (
    <section className="cue-page">
      <header className="crud-header">
        <div>
          <p className="panel-kicker">Cue Editor</p>
          <h2>{waveform ? trackTitle(waveform.track) : `Track #${trackId}`}</h2>
        </div>
        <a className="ghost-button" href="/tracks">Back</a>
      </header>

      {error ? <div className="table-error">{error}</div> : null}
      {loading ? <div className="table-loading">Generating waveform...</div> : null}

      <div className="waveform-panel">
        <canvas ref={canvasRef} className="waveform-canvas" />
      </div>

      <audio
        ref={audioRef}
        preload="metadata"
        src={`/api/tracks/${trackId}/audio`}
        onEnded={() => setPlaying(false)}
        onPause={() => setPlaying(false)}
        onPlay={() => setPlaying(true)}
        onTimeUpdate={(event) => setCurrentTime(event.currentTarget.currentTime)}
      />

      <div className="transport-bar">
        <button className="primary-button transport-button" aria-label="Play" disabled={!waveform} title="Play" type="button" onClick={() => play(audioRef.current)}>
          <i className="bi bi-play-fill" aria-hidden="true" />
        </button>
        <button className="ghost-button transport-button" aria-label="Stop" disabled={!waveform} title="Stop" type="button" onClick={() => stop(audioRef.current)}>
          <i className="bi bi-stop-fill" aria-hidden="true" />
        </button>
        <button className="ghost-button transport-button" aria-label="Back 5 seconds" disabled={!waveform} title="Back 5 seconds" type="button" onClick={() => seekBy(audioRef.current, -5)}>
          <i className="bi bi-skip-backward-fill" aria-hidden="true" />
        </button>
        <button className="ghost-button transport-button" aria-label="Forward 5 seconds" disabled={!waveform} title="Forward 5 seconds" type="button" onClick={() => seekBy(audioRef.current, 5)}>
          <i className="bi bi-skip-forward-fill" aria-hidden="true" />
        </button>
        <div className="cue-button-group">
          {CUE_MARKERS.map((marker) => (
            <button
              className="cue-marker-button"
              disabled={!waveform || savingMarker === marker.field}
              key={marker.field}
              style={{ '--marker-color': marker.color }}
              type="button"
              onClick={() => markCuePoint(marker.field)}
            >
              {marker.label}
            </button>
          ))}
        </div>
        <span className={playing ? 'transport-status playing' : 'transport-status'}>
          {playing ? 'Playing' : 'Stopped'} · {formatDuration(currentTime)}
        </span>
      </div>

      {waveform ? (
        <div className="cue-meta">
          <span>Duration: <strong>{formatDuration(waveform.duration)}</strong></span>
          <span>Sample Rate: <strong>{waveform.sampleRate} Hz</strong></span>
          <span>Channels: <strong>{waveform.channels}</strong></span>
          <span>Peaks: <strong>{waveform.peaks.length}</strong></span>
        </div>
      ) : null}
    </section>
  );

  async function markCuePoint(field) {
    if (!audioRef.current || !waveformRef.current) return;

    const value = Math.max(0, audioRef.current.currentTime || 0);
    setSavingMarker(field);
    setError(null);

    try {
      const response = await fetch(`/api/tracks/${trackId}/cue-point`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          field,
          value
        })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(payload.error || `Request failed with status ${response.status}`);

      setWaveform((current) => {
        if (!current) return current;
        const next = {
          ...current,
          track: {
            ...current.track,
            cue_points: payload.cue_points
          }
        };
        waveformRef.current = next;
        drawCueCanvas(canvasRef.current, next, audioRef.current);
        return next;
      });
    } catch (err) {
      setError(err.message);
    } finally {
      setSavingMarker('');
    }
  }
}

async function play(audio) {
  if (!audio) return;
  await audio.play();
}

function stop(audio) {
  if (!audio) return;
  audio.pause();
  audio.currentTime = 0;
}

function seekBy(audio, seconds) {
  if (!audio) return;
  const duration = Number.isFinite(audio.duration) ? audio.duration : Infinity;
  audio.currentTime = Math.min(duration, Math.max(0, audio.currentTime + seconds));
}

function drawCueCanvas(canvas, waveform, audio) {
  if (!waveform) return;
  const audioDuration = Number.isFinite(audio?.duration) ? audio.duration : waveform.duration;
  const ratio = playbackRatio(audio?.currentTime || 0, audioDuration || waveform.duration);
  drawWaveform(canvas, waveform, ratio);
}

function drawWaveform(canvas, waveform, playheadRatio = 0) {
  const peaks = waveform?.peaks || [];
  if (!canvas || !peaks?.length) return;

  const rect = canvas.getBoundingClientRect();
  const pixelRatio = window.devicePixelRatio || 1;
  const width = Math.max(1, Math.floor(rect.width * pixelRatio));
  const height = Math.max(1, Math.floor(rect.height * pixelRatio));

  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;

  const ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = '#f5f8fb';
  ctx.fillRect(0, 0, width, height);

  const mid = height / 2;
  ctx.strokeStyle = '#d9e2e8';
  ctx.lineWidth = Math.max(1, pixelRatio);
  ctx.beginPath();
  ctx.moveTo(0, mid);
  ctx.lineTo(width, mid);
  ctx.stroke();

  ctx.strokeStyle = '#19005a';
  ctx.lineWidth = Math.max(1, pixelRatio);
  ctx.beginPath();

  for (let x = 0; x < width; x += 1) {
    const peakIndex = Math.min(peaks.length - 1, Math.floor((x / width) * peaks.length));
    const [min, max] = peaks[peakIndex];
    const y1 = mid + min * mid * 0.92;
    const y2 = mid + max * mid * 0.92;
    ctx.moveTo(x + 0.5, y1);
    ctx.lineTo(x + 0.5, y2);
  }

  ctx.stroke();

  drawCueMarkers(ctx, waveform.track?.cue_points || {}, waveform.duration, width, height, pixelRatio);

  const playheadX = Math.max(0, Math.min(1, playheadRatio)) * width;
  ctx.strokeStyle = '#ff1d25';
  ctx.lineWidth = Math.max(2, 2 * pixelRatio);
  ctx.beginPath();
  ctx.moveTo(playheadX, 0);
  ctx.lineTo(playheadX, height);
  ctx.stroke();
}

function drawCueMarkers(ctx, cuePoints, duration, width, height, pixelRatio) {
  const total = Number(duration || 0);
  if (!Number.isFinite(total) || total <= 0) return;

  for (const marker of CUE_MARKERS) {
    const value = Number(cuePoints?.[marker.field]);
    if (!Number.isFinite(value) || value <= 0) continue;

    const x = Math.max(0, Math.min(1, value / total)) * width;
    ctx.strokeStyle = marker.color;
    ctx.lineWidth = Math.max(2, 2 * pixelRatio);
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }
}

function playbackRatio(currentTime, duration) {
  const time = Number(currentTime || 0);
  const total = Number(duration || 0);
  if (!Number.isFinite(time) || !Number.isFinite(total) || total <= 0) return 0;
  return time / total;
}

function trackTitle(track) {
  return [track.artist, track.title].filter(Boolean).join(' - ') || `Track #${track.id}`;
}

function formatDuration(seconds) {
  const total = Math.max(0, Math.round(Number(seconds || 0)));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}
