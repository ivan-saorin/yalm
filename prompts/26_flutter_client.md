# PROMPT 26 — Flutter Client (Voice + DAFHNE on Device)

> **STATUS: Stub. Mobile/tablet app — old phone becomes a home terminal.**
>
> **Depends on**: Phase 22 (server), Phase 23 (layered dictionary), Phase 25 (chat-as-config)
> **Optional**: Phase 24 (packages) for package management UI

## GOAL

A Flutter app that turns any Android phone/tablet into a DAFHNE terminal. Always-on display, voice activated, runs DAFHNE engine locally for instant answers, syncs dictionary from server. The phone sits on a shelf and becomes a home assistant.

## ARCHITECTURE

```
┌─────────────────────────────────────┐
│          Flutter App                 │
│                                      │
│  ┌──────────┐  ┌─────────────────┐  │
│  │ Voice UI  │  │  Chat UI        │  │
│  │ STT → TTS │  │  (text fallback)│  │
│  └─────┬─────┘  └───────┬────────┘  │
│        │                │            │
│        └────────┬───────┘            │
│                 ▼                    │
│  ┌──────────────────────────────┐   │
│  │  DAFHNE Engine (Rust via FFI)│   │
│  │  Local geometry, instant     │   │
│  │  No network needed for Q&A   │   │
│  └──────────────┬───────────────┘   │
│                 │                    │
│  ┌──────────────▼───────────────┐   │
│  │  Sync Layer                   │   │
│  │  Dictionary updates from srv  │   │
│  │  Action dispatch to server    │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
         │              ▲
         ▼              │
    dafhne-server   (dict sync + action execution)
```

## RUST ↔ FLUTTER BRIDGE

Use `flutter_rust_bridge` (v2) for FFI code generation.

### Rust API exposed to Dart

```rust
// In a new crate: dafhne-ffi or dafhne-mobile

/// Initialize engine from serialized space (downloaded from server)
pub fn init_from_space(space_bytes: &[u8]) -> Result<EngineHandle, String>;

/// Initialize engine from dictionary text (offline bootstrap)
pub fn init_from_dict(dict_text: &str, genome_json: &str) -> Result<EngineHandle, String>;

/// Ask a question — returns answer string
pub fn ask(handle: &EngineHandle, question: &str) -> String;

/// Describe a word
pub fn describe(handle: &EngineHandle, word: &str) -> String;

/// List all known words
pub fn list_words(handle: &EngineHandle) -> Vec<String>;

/// Check if engine knows a word
pub fn knows(handle: &EngineHandle, word: &str) -> bool;

/// Get engine version/dictionary hash (for sync check)
pub fn dict_version(handle: &EngineHandle) -> String;

/// Hot-swap engine with new space (after sync)
pub fn reload_space(handle: &mut EngineHandle, space_bytes: &[u8]) -> Result<(), String>;
```

### New crate structure

```
crates/
├── dafhne-core/
├── dafhne-engine/
├── dafhne-server/
├── dafhne-eval/
├── dafhne-evolve/
└── dafhne-ffi/          # NEW — thin FFI wrapper
    ├── Cargo.toml       # depends on dafhne-engine
    └── src/
        └── lib.rs       # pub functions for flutter_rust_bridge
```

`dafhne-ffi` compiles to:
- `libdafhne.so` (Android arm64/armv7)
- `libdafhne.dylib` (iOS — if ever needed)
- `dafhne.dll` (Windows desktop — for testing)

## FLUTTER APP STRUCTURE

```
flutter_dafhne/
├── lib/
│   ├── main.dart
│   ├── screens/
│   │   ├── home_screen.dart        # main always-on display
│   │   ├── chat_screen.dart        # text chat fallback
│   │   └── settings_screen.dart    # server URL, voice settings
│   ├── services/
│   │   ├── dafhne_service.dart     # Rust bridge wrapper
│   │   ├── voice_service.dart      # STT + TTS
│   │   ├── sync_service.dart       # server dictionary sync
│   │   └── action_service.dart     # forward actions to server
│   ├── widgets/
│   │   ├── listening_indicator.dart # visual "I'm listening" animation
│   │   ├── answer_bubble.dart      # shows answer with reasoning
│   │   └── vocabulary_chip.dart    # shows known words
│   └── models/
│       └── config.dart
├── rust/                           # flutter_rust_bridge generated
│   └── src/
│       └── api.rs                  # auto-generated from dafhne-ffi
├── android/
├── ios/
└── pubspec.yaml
```

## VOICE PIPELINE

### Speech-to-Text (STT)

```dart
// Using speech_to_text package
class VoiceService {
  final SpeechToText _stt = SpeechToText();
  
  // Wake word detection
  // Option A: Always listening, detect "Dafhne" prefix
  // Option B: Push-to-talk button on screen
  // Option C: Android voice activity detection
  
  Future<void> startListening() async {
    await _stt.listen(
      onResult: (result) {
        if (result.finalResult) {
          _processUtterance(result.recognizedWords);
        }
      },
      listenFor: Duration(seconds: 10),
      localeId: 'en_US',  // or detect from system
    );
  }
}
```

### Text-to-Speech (TTS)

```dart
// Using flutter_tts package
class TtsService {
  final FlutterTts _tts = FlutterTts();
  
  Future<void> init() async {
    await _tts.setLanguage('en-US');
    await _tts.setSpeechRate(0.5);   // slightly slow for clarity
    await _tts.setVolume(1.0);
    await _tts.setPitch(1.0);
  }
  
  Future<void> speak(String text) async {
    await _tts.speak(text);
  }
}
```

### Wake Word

Options ranked by practicality:

1. **"Hey Dafhne" / "Dafhne"** — use `speech_to_text` continuous mode, check if first word is "dafhne" variant. Simple but battery-hungry.

2. **Porcupine / Picovoice** — dedicated on-device wake word engine. Very low power. Free tier available. Flutter plugin exists. Custom wake word "Dafhne" needs their console to train.

3. **Push-to-talk** — screen button. No wake word needed. Best for old phone on a stand — just tap and talk.

4. **Proximity sensor** — wave hand near phone to activate. Some Android phones support this. Quirky but cool.

Recommendation: Start with push-to-talk (simplest), add Picovoice wake word as an option later.

## ALWAYS-ON DISPLAY

The phone sits on a shelf. The screen shows:

```
┌─────────────────────────┐
│                         │
│      ◇ DAFHNE           │
│      ready               │
│                         │
│   "Is the light on?"    │
│   → Yes.                │
│                         │
│   ┌───────────────────┐ │
│   │  🎤  Tap to talk  │ │
│   └───────────────────┘ │
│                         │
│  ⚡ 287 words │ 🔗 synced│
└─────────────────────────┘
```

When listening:
```
┌─────────────────────────┐
│                         │
│      ◇ DAFHNE           │
│      listening...       │
│                         │
│   ┌─ ─ ─ ─ ─ ─ ─ ─ ─┐ │
│   │  ≋≋≋≋≋≋≋≋≋≋≋≋≋≋  │ │ ← waveform animation
│   └─ ─ ─ ─ ─ ─ ─ ─ ─┘ │
│                         │
│                         │
│                         │
└─────────────────────────┘
```

When answering:
```
┌─────────────────────────┐
│                         │
│      ◇ DAFHNE           │
│                         │
│   You: Is a dog a food? │
│                         │
│   ◇ No.                 │
│   [content · 0.3ms]     │
│                         │
│   ┌───────────────────┐ │
│   │  🎤  Tap to talk  │ │
│   └───────────────────┘ │
│                         │
└─────────────────────────┘
```

### Screen management

- `Wakelock` plugin — keep screen on while plugged in
- Dim screen after 30s inactivity (save OLED)
- Brighten on voice activity or touch
- Show clock when idle (useful as bedside terminal)

## DICTIONARY SYNC

```dart
class SyncService {
  final String serverUrl;
  Timer? _pollTimer;
  
  Future<void> startSync() async {
    // 1. Initial full sync on app start
    await fullSync();
    
    // 2. Periodic version check (every 30s)
    _pollTimer = Timer.periodic(Duration(seconds: 30), (_) => checkVersion());
    
    // 3. WebSocket for instant push (if server supports)
    // _connectWebSocket();
  }
  
  Future<void> checkVersion() async {
    final serverVersion = await http.get('$serverUrl/api/dictionary/version');
    final localVersion = dafhneService.dictVersion();
    
    if (serverVersion != localVersion) {
      await fullSync();
    }
  }
  
  Future<void> fullSync() async {
    // Download pre-built space (NOT raw dictionary — skip training on device)
    final spaceBytes = await http.getBytes('$serverUrl/api/dictionary/space');
    dafhneService.reloadSpace(spaceBytes);
  }
}
```

## ACTION FORWARDING

The client comprehends locally but executes remotely. When DAFHNE resolves an action:

```dart
// Local comprehension (instant, no network)
final result = dafhneService.ask("turn on the kitchen light");
// result.type = Action
// result.action = "turn-on"  
// result.target = "kitchen-light"

// Forward to server for execution
await http.post('$serverUrl/api/actions/execute', body: {
  'action': result.action,
  'target': result.target,
  'params': result.params,
});
```

Comprehension: local, instant, offline-capable.
Execution: server-side, needs network, adapter handles it.

## CONFIG VIA VOICE

Phase 25's chat-as-config, but spoken:

```
You:    "Dafhne add bedroom lamp as a light in the bedroom"
DAFHNE: [resolves locally: this is a config command]
        [forwards to server: POST /api/config/add]
        [server validates, adds to user dict, rebuilds]
        [client syncs new space]
        "Done. I added bedroom lamp."
        
You:    "Is the bedroom lamp in the kitchen?"
DAFHNE: [answers locally from updated geometry]
        "No."
```

## MULTI-LANGUAGE SUPPORT

Android STT/TTS support 100+ languages. DAFHNE's engine is language-agnostic — the dictionaries are English but the geometry doesn't care. A future Italian core dictionary would work identically:

```
cane — un animale che vive con le persone
gatto — un animale piccolo che fa le fusa
```

The Flutter app detects system locale and configures STT/TTS language. The server serves the appropriate dictionary. Everything else is identical.

## HARDWARE TARGETS

| Device | Role | Notes |
|--------|------|-------|
| Old Android phone (API 24+) | Primary | On shelf, always on, push-to-talk |
| Android tablet | Kitchen display | Bigger screen, recipe mode? |
| Raspberry Pi + touchscreen | Dedicated terminal | Rust native, no Flutter needed |
| Desktop (Windows/Mac/Linux) | Dev/testing | Flutter desktop build |

Minimum Android: API 24 (Android 7.0, 2016). Rust cross-compiles to armv7 and arm64.

## BUILD PIPELINE

```bash
# 1. Build Rust FFI library for Android targets
cd crates/dafhne-ffi
cargo ndk -t armeabi-v7a -t arm64-v8a build --release

# 2. Generate Dart bindings
cd flutter_dafhne
flutter_rust_bridge_codegen generate

# 3. Build Flutter app
flutter build apk --release
```

## TESTING

- Engine loads from serialized space → instant init, words queryable
- STT → DAFHNE → TTS round trip under 500ms (excluding STT latency)
- Dictionary sync: change word on server → client picks up within 30s
- Offline mode: disconnect network, Q&A still works, actions queued
- Config via voice: "add X as Y" → server processes, client syncs
- Always-on: 24h battery test while plugged in, no crashes
- Multiple clients: 3 phones synced to same server, all see same vocabulary

## WHAT NOT TO DO

- Do NOT run engine training on the phone (download pre-built space from server)
- Do NOT require internet for Q&A (only for sync and actions)
- Do NOT build iOS version yet (Android only — old phones are Android)
- Do NOT implement complex voice UX (keep it simple: listen, answer, done)
- Do NOT build custom wake word model (use push-to-talk or existing libraries)
- Do NOT handle music/media playback on the phone itself (forward to server/adapter)
