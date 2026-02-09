# PROMPT 24 — Package System + Adapters

> **STATUS: Stub. Installable vocabulary packages with action bindings.**
>
> **Depends on**: Phase 23 (layered dictionary)
> **Enables**: Phase 25 (chat-as-config), Phase 26 (Flutter client)

## GOAL

Define the `.pkg.toml` format for vocabulary packages. Implement install/remove. Separate comprehension (dictionary) from execution (adapters). A package teaches DAFHNE words; an adapter teaches it how to act on them.

## PACKAGE FORMAT

```toml
[package]
name = "music"
version = "1.0.0"
description = "Music playback — play, pause, skip, playlists"
author = "dafhne-community"
requires_core = ">=1.0"
depends = []                    # other packages this one needs
space = "content"               # which DAFHNE space these words join
                                # or "auto" to let the engine decide

[dictionary]
# ELI5 definitions — every word must close over core + depends + this dict
song = "a thing that is music with words"
album = "a group of songs by one artist"
playlist = "a group of songs a person picks"
artist = "a person who makes music"
play = "a way to make music start"
pause = "a way to make music stop for now"
skip = "a way to go to the next song"
shuffle = "a way to play songs in a random order"
volume = "how loud a thing is"
track = "one song in an album or playlist"
genre = "a kind of music"
lyrics = "the words in a song"

[entities]
# Bindable services — these become dictionary entries automatically
# "spotify — a music place on the internet"
spotify = { definition = "a music place on the internet" }
apple-music = { definition = "a music place on the internet" }
amazon-music = { definition = "a music place on the internet" }
youtube-music = { definition = "a music place on the internet" }

[actions]
# Abstract action → route mapping
# {target} = resolved entity name, {param} = quoted string passthrough
"play" = { 
    applies_to = ["song", "album", "playlist", "artist"],
    route = "/music/play",
    params = { query = "{param}", service = "{target}" }
}
"pause" = {
    applies_to = ["song"],
    route = "/music/pause",
    params = { service = "{target}" }
}
"skip" = {
    applies_to = ["song"],
    route = "/music/skip",
    params = { service = "{target}" }
}
"set.volume" = {
    applies_to = ["volume"],
    route = "/music/volume",
    params = { level = "{param}", service = "{target}" }
}

[test]
# Built-in package validation questions
"Is a song a thing?" = "Yes"
"Is spotify a music place?" = "Yes"
"What is an album?" = "a group of songs by one artist"
```

## ADAPTER FORMAT

Adapters are separate from packages. A package defines abstract routes; an adapter maps routes to real APIs.

```toml
# adapters/spotify.adapter.toml
[adapter]
name = "spotify"
package = "music"               # which package this adapts
entity = "spotify"              # which entity this binds to
version = "1.0.0"

[auth]
type = "oauth2"
client_id = ""                  # user fills in
client_secret = ""              # user fills in
token_url = "https://accounts.spotify.com/api/token"
scopes = ["user-modify-playback-state", "user-read-playback-state"]

[routes]
# Map abstract routes to concrete API calls
"/music/play" = {
    method = "PUT",
    url = "https://api.spotify.com/v1/me/player/play",
    headers = { "Authorization" = "Bearer {token}" },
    body = '{"uris":["spotify:search:{query}"]}'
}
"/music/pause" = {
    method = "PUT",
    url = "https://api.spotify.com/v1/me/player/pause",
    headers = { "Authorization" = "Bearer {token}" }
}
"/music/skip" = {
    method = "POST",
    url = "https://api.spotify.com/v1/me/player/next",
    headers = { "Authorization" = "Bearer {token}" }
}
"/music/volume" = {
    method = "PUT",
    url = "https://api.spotify.com/v1/me/player/volume?volume_percent={level}",
    headers = { "Authorization" = "Bearer {token}" }
}
```

## THE SEPARATION

```
"Dafhne play 'Yesterday' on Spotify"
        │
        ▼
   DAFHNE Engine (comprehension)
   → action: play
   → param: "Yesterday" (opaque passthrough)
   → entity: spotify (resolved to music-place)
        │
        ▼
   Package Router (music.pkg)
   → route: /music/play
   → params: { query: "Yesterday", service: "spotify" }
        │
        ▼
   Adapter Executor (spotify.adapter)
   → PUT https://api.spotify.com/v1/me/player/play
   → body: {"uris":["spotify:search:Yesterday"]}
        │
        ▼
   Spotify plays Yesterday
```

Three actors, three responsibilities:
- **Engine**: understands the sentence (geometric)
- **Package**: maps understanding to abstract route (declarative)
- **Adapter**: maps abstract route to real API (imperative)

## PACKAGE LIFECYCLE

### Install
```
POST /api/packages/install
body: { "name": "music" }  # from registry or local file

1. Parse package.toml
2. Verify depends are installed
3. Validate dictionary closure against (core + installed packages)
4. Merge into LayeredDictionary (Phase 23)
5. Rebuild geometry
6. Persist to packages/ directory
```

### Remove
```
POST /api/packages/remove
body: { "name": "music" }

1. Check no other package depends on this
2. Check no user words depend on package words
3. Remove from LayeredDictionary
4. Remove associated adapters
5. Rebuild geometry
6. Persist removal
```

### Update
```
POST /api/packages/update
body: { "name": "music", "version": "1.1.0" }

1. Download new version
2. Diff: new words, changed definitions, removed words
3. Check user overrides: warn if package changed an overridden word
4. Check closure of new version
5. Swap in place, rebuild geometry
```

## FIRST-PARTY PACKAGES TO CREATE

| Package | Words | Entities | Actions |
|---------|-------|----------|---------|
| `music` | ~15 | spotify, apple-music, amazon-music | play, pause, skip, volume |
| `lights` | ~10 | (user adds specific lights) | turn-on, turn-off, dim, color |
| `thermostat` | ~8 | (user adds specific devices) | set-temp, mode |
| `timer` | ~8 | — | set-timer, cancel-timer |
| `weather` | ~12 | — | (read-only, no actions) |
| `calendar` | ~12 | google-calendar, outlook | add-event, list-events |
| `messaging` | ~8 | whatsapp, telegram | send-message |

## WORD COLLISION RESOLUTION

"Volume" in music (loudness) vs "volume" in math (space inside a shape):

Option A: Namespace — `music:volume` vs `math:volume`. Ugly, breaks natural language.

Option B: Multi-space handles it — music-volume lives in content space near "loud", math-volume lives in math space near "number". Query routing picks the right one based on context. "Set the volume to 5" → music (because "set" + entity context). "What is the volume of a box?" → math.

**Option B is the DAFHNE way.** The geometry disambiguates. Different spaces, different positions, different meanings. Same word, different understanding. This is how human language works.

## API ENDPOINTS

```
GET  /api/packages                    → list installed packages
GET  /api/packages/{name}             → package details + word list
POST /api/packages/install            → install from registry or upload
POST /api/packages/remove             → uninstall
POST /api/packages/update             → update to new version

GET  /api/adapters                    → list configured adapters
GET  /api/adapters/{name}             → adapter config (sans secrets)
POST /api/adapters/{name}/configure   → set credentials/endpoints
POST /api/adapters/{name}/test        → test adapter connectivity

POST /api/actions/resolve             → dry-run: parse sentence, show what would execute
POST /api/actions/execute             → parse + execute through adapter
```

## PACKAGE REGISTRY (future)

Like crates.io but for DAFHNE vocabulary packages. Out of scope for Phase 24 — install from local files only. But design the package format to be registry-ready.

## TESTING

- Install music package → new words appear in vocabulary
- "Is a song a thing?" → Yes (through package dictionary)
- "Play 'Yesterday' on Spotify" → resolves to correct route + params
- Remove music package → words disappear, user words using music terms fail closure check
- Collision: two packages define same word → space routing disambiguates
- Package self-test: run [test] section after install

## WHAT NOT TO DO

- Do NOT build a registry server (local install only)
- Do NOT implement OAuth flows (adapter config is manual for now)
- Do NOT build actual Spotify/Hue/etc integrations (just the framework)
- Do NOT modify DAFHNE engine — packages feed into existing dictionary system
