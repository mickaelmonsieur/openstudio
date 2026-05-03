const THRESHOLD_DBFS = -40;
const THRESHOLD_LINEAR = Math.pow(10, THRESHOLD_DBFS / 20); // ≈ 0.01
const WINDOW_SECONDS = 0.01;    // 10ms per analysis window
const MIN_DURATION_SECONDS = 0.2; // 200ms of sustained audio required
const SCAN_DURATION_SECONDS = 30; // only scan first/last 30s

export function detectCuePoints(channelData, sampleRate, samplesDecoded) {
  const windowSize = Math.max(1, Math.round(sampleRate * WINDOW_SECONDS));
  const minWindows = Math.ceil(MIN_DURATION_SECONDS / WINDOW_SECONDS);
  const scanSamples = Math.min(samplesDecoded, Math.round(sampleRate * SCAN_DURATION_SECONDS));

  const cue_in = detectCueIn(channelData, sampleRate, windowSize, minWindows, scanSamples);
  const cue_out = detectCueOut(channelData, sampleRate, samplesDecoded, windowSize, minWindows, scanSamples);

  return { cue_in, cue_out };
}

function detectCueIn(channelData, sampleRate, windowSize, minWindows, scanSamples) {
  let consecutive = 0;
  let candidateStart = 0;

  for (let pos = 0; pos < scanSamples; pos += windowSize) {
    const end = Math.min(scanSamples, pos + windowSize);
    if (windowRms(channelData, pos, end) >= THRESHOLD_LINEAR) {
      if (consecutive === 0) candidateStart = pos;
      consecutive++;
      if (consecutive >= minWindows) {
        return round3(candidateStart / sampleRate);
      }
    } else {
      consecutive = 0;
    }
  }
  return 0;
}

function detectCueOut(channelData, sampleRate, samplesDecoded, windowSize, minWindows, scanSamples) {
  let consecutive = 0;
  let candidateEnd = samplesDecoded;
  const scanStart = samplesDecoded - scanSamples;

  for (let pos = samplesDecoded - windowSize; pos >= scanStart; pos -= windowSize) {
    const start = Math.max(scanStart, pos);
    const end = Math.min(samplesDecoded, pos + windowSize);
    if (windowRms(channelData, start, end) >= THRESHOLD_LINEAR) {
      if (consecutive === 0) candidateEnd = end;
      consecutive++;
      if (consecutive >= minWindows) {
        return round3(candidateEnd / sampleRate);
      }
    } else {
      consecutive = 0;
    }
  }
  return null;
}

function windowRms(channelData, start, end) {
  let sum = 0;
  let count = 0;
  for (const channel of channelData) {
    for (let i = start; i < end; i++) {
      const v = channel[i] || 0;
      sum += v * v;
      count++;
    }
  }
  return count > 0 ? Math.sqrt(sum / count) : 0;
}

function round3(value) {
  return Math.round(value * 1000) / 1000;
}
