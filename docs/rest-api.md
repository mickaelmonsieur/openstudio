# OpenStudio Player REST API

The OpenStudio Rust player exposes a local playback-control REST API.

- Base URL: `http://<host>:7080/api/v1`
- Default bind address: `0.0.0.0:7080`
- Authentication: none
- CORS: not handled
- IDs in routes are 1-based, matching the visible player buttons.
- Mutating endpoints return the same envelope as `GET /status`, with an updated `status` snapshot.

## Response Envelope

```json
{
  "ok": true,
  "message": "Deck play",
  "status": {}
}
```

Errors use the same envelope without `status`:

```json
{
  "ok": false,
  "message": "Instant slot is empty."
}
```

## Status

```http
GET /api/v1/status
```

Returns automix, deck, preview, instant, aux, and queue state.

Example:

```sh
curl http://127.0.0.1:7080/api/v1/status
```

## Automix

```http
PUT /api/v1/automix
Content-Type: application/json

{ "enabled": true }
```

Example:

```sh
curl -X PUT http://127.0.0.1:7080/api/v1/automix \
  -H 'Content-Type: application/json' \
  -d '{"enabled":true}'
```

## Deck Transport

```http
POST /api/v1/deck/play
POST /api/v1/deck/pause
POST /api/v1/deck/play-pause
POST /api/v1/deck/stop
POST /api/v1/deck/restart
```

Seek is relative, in milliseconds:

```http
POST /api/v1/deck/seek
Content-Type: application/json

{ "offset_ms": 5000 }
```

Use a negative value to rewind:

```sh
curl -X POST http://127.0.0.1:7080/api/v1/deck/seek \
  -H 'Content-Type: application/json' \
  -d '{"offset_ms":-5000}'
```

## Deck Direct Queue Buttons

Start a visible queue item immediately:

```http
POST /api/v1/deck/queue/{id}/play
```

`id` is the visible queue position, starting at 1.

Example:

```sh
curl -X POST http://127.0.0.1:7080/api/v1/deck/queue/1/play
```

## Deck Preview

Start or toggle preview for a visible queue item:

```http
POST /api/v1/deck/queue/{id}/preview/play
POST /api/v1/deck/queue/{id}/preview/toggle
```

Stop preview:

```http
POST /api/v1/deck/preview/stop
```

Seek preview:

```http
POST /api/v1/deck/preview/seek
Content-Type: application/json

{ "offset_ms": 5000 }
```

## Instant Player

The current implementation has 10 Instant slots.

```http
POST /api/v1/instant/{id}/play
POST /api/v1/instant/{id}/stop
```

Set loop state for a slot:

```http
PUT /api/v1/instant/{id}/loop
Content-Type: application/json

{ "enabled": true }
```

Loop state is live playback state and is not persisted in Instant pages.

Examples:

```sh
curl -X POST http://127.0.0.1:7080/api/v1/instant/1/play
curl -X PUT http://127.0.0.1:7080/api/v1/instant/1/loop \
  -H 'Content-Type: application/json' \
  -d '{"enabled":true}'
```

## Aux Players

The current implementation has 3 AUX players.

```http
POST /api/v1/aux/{id}/play
POST /api/v1/aux/{id}/stop
```

Set loop state:

```http
PUT /api/v1/aux/{id}/loop
Content-Type: application/json

{ "enabled": true }
```

Examples:

```sh
curl -X POST http://127.0.0.1:7080/api/v1/aux/1/play
curl -X PUT http://127.0.0.1:7080/api/v1/aux/1/loop \
  -H 'Content-Type: application/json' \
  -d '{"enabled":false}'
```

## Notes

The HTTP server runs in a separate thread. Incoming REST commands are forwarded to the Iced application loop, and all player state mutations still happen on the main application side.

The deck is exposed as one logical deck, even though OpenStudio internally uses two queue players for fade and automix behavior.
