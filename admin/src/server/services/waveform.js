import fs from 'node:fs/promises';
import { FLACDecoder } from '@wasm-audio-decoders/flac';

const waveformCache = new Map();
const MAX_CACHE_ITEMS = 20;

export async function generateWaveform(track, options = {}) {
  if (!track?.path) {
    throw new Error('Track file path is missing.');
  }

  const points = clampInteger(options.points, 400, 12000, 2400);
  const stats = await fs.stat(track.path);
  const cacheKey = `${track.path}:${stats.mtimeMs}:${stats.size}:${points}`;

  if (waveformCache.has(cacheKey)) {
    const cachedAudio = waveformCache.get(cacheKey);
    waveformCache.delete(cacheKey);
    waveformCache.set(cacheKey, cachedAudio);
    return buildWaveformResponse(track, points, cachedAudio);
  }

  const fileData = await fs.readFile(track.path);
  const decoder = new FLACDecoder();
  await decoder.ready;
  const decoded = await decoder.decodeFile(new Uint8Array(fileData));

  const audio = {
    sampleRate: decoded.sampleRate,
    channels: decoded.channelData.length,
    samples: decoded.samplesDecoded,
    duration: decoded.samplesDecoded / decoded.sampleRate,
    peaks: buildPeaks(decoded.channelData, decoded.samplesDecoded, points)
  };

  remember(cacheKey, audio);
  return buildWaveformResponse(track, points, audio);
}

function buildWaveformResponse(track, points, audio) {
  return {
    track: {
      id: track.id,
      artist: track.artist,
      title: track.title,
      album: track.album,
      year: track.year,
      path: track.path,
      cue_points: {
        cue_in: nullableNumber(track.cue_in),
        intro: nullableNumber(track.intro),
        hook_in: nullableNumber(track.hook_in),
        hook_out: nullableNumber(track.hook_out),
        loop_in: nullableNumber(track.loop_in),
        loop_out: nullableNumber(track.loop_out),
        outro: nullableNumber(track.outro),
        cue_out: nullableNumber(track.cue_out)
      }
    },
    sampleRate: audio.sampleRate,
    channels: audio.channels,
    samples: audio.samples,
    duration: audio.duration,
    points,
    peaks: audio.peaks
  };
}

function nullableNumber(value) {
  if (value === null || value === undefined) return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function buildPeaks(channelData, samplesDecoded, points) {
  const bucketSize = Math.max(1, Math.ceil(samplesDecoded / points));
  const peaks = [];

  for (let start = 0; start < samplesDecoded; start += bucketSize) {
    const end = Math.min(samplesDecoded, start + bucketSize);
    let min = 0;
    let max = 0;

    for (const channel of channelData) {
      for (let index = start; index < end; index += 1) {
        const value = channel[index] || 0;
        if (value < min) min = value;
        if (value > max) max = value;
      }
    }

    peaks.push([
      roundPeak(Math.max(-1, min)),
      roundPeak(Math.min(1, max))
    ]);
  }

  return peaks;
}

function roundPeak(value) {
  return Math.round(value * 10000) / 10000;
}

function clampInteger(value, min, max, fallback) {
  const number = Number(value);
  if (!Number.isInteger(number)) return fallback;
  return Math.min(max, Math.max(min, number));
}

function remember(key, value) {
  waveformCache.set(key, value);
  while (waveformCache.size > MAX_CACHE_ITEMS) {
    const oldest = waveformCache.keys().next().value;
    waveformCache.delete(oldest);
  }
}
