# EC Extension: OpenFX (`ec-extension-ofx`)

> [!WARNING]
> **Unofficial.** This is not part of [EffectCraft](https://github.com/storytold/effectcraft) and
> isn't supported by its maintainers or by any plug-in vendor. Please report problems in
> [this repository's Issues](https://github.com/glumbis/ec-extension-ofx/issues).

Run **OpenFX (OFX) plug-ins** in [EffectCraft](https://github.com/storytold/effectcraft) as
ordinary effects: RE:Vision **ReelSmart Motion Blur** and **Twixtor**, Boris FX **Sapphire**, the
free OFX collections and other plug-ins with a CPU render path. They show up in Effects &
Presets, their settings appear in Effect Controls, and they take keyframes and expressions.

This is an **external effect helper**: a small program, `effectcraft-helper-ofx`, that EffectCraft
starts next to itself. The plug-ins run inside the helper, never inside EffectCraft. **If a plug-in
crashes, EffectCraft keeps running**: it shows ⚠ on the effect, leaves the frame as it was and
starts the helper again for the next frame.

```
EffectCraft ──(pipe: small JSON messages)──▶ effectcraft-helper-ofx ──▶ your .ofx plug-ins
      ▲                                              │
      └──────(shared frame file: the pixels)─────────┘
```

| Plug-in | Status |
|---|---|
| RE:Vision ReelSmart Motion Blur (RSMB, Pro, Pro Vectors) | loads and renders (in-process build); retest through the helper pending |
| RE:Vision Twixtor (Twixtor, Pro, Vectors In, Motion Vectors Create) | slow motion and speed ramps (in-process build); retest pending |
| Boris FX Sapphire (~290 effects) | loads and renders (in-process build); retest pending |
| `ec-test-ofx` (in this repo) | tested in CI on Windows, Linux and macOS through the helper |

## What you need

- An EffectCraft build with **external effect helper** support. It's proposed upstream (branch
  [`claude/openfx-external-helper-jdpn38`](https://github.com/glumbis/effectcraft/tree/claude/openfx-external-helper-jdpn38));
  until it is merged, build EffectCraft from that branch.
- The **OFX** versions of your plug-ins, installed with their own installers.

## Install

1. **Get the helper.** Download `effectcraft-helper-ofx` for your system from
   [Releases](https://github.com/glumbis/ec-extension-ofx/releases) (or the artifacts of the latest
   [CI run](https://github.com/glumbis/ec-extension-ofx/actions)), and check it against
   `SHA256SUMS.txt`. Or build it yourself (below).
2. **Put it where EffectCraft looks**, any one of:
   - a `helpers` folder next to the EffectCraft program (`effectcraft.exe`);
   - `helpers` in EffectCraft's settings folder: `%APPDATA%\EffectCraft\helpers` (Windows),
     `~/Library/Application Support/EffectCraft/helpers` (macOS), `~/.config/effectcraft/helpers`
     (Linux);
   - any folder you list in the `EFFECTCRAFT_HELPER_PATH` environment variable.
3. **Install your plug-ins' OFX versions.** They go to the standard OFX folder, where the helper
   finds them:

   | OS | Standard folder |
   |---|---|
   | Windows | `C:\Program Files\Common Files\OFX\Plugins` |
   | macOS | `/Library/OFX/Plugins` |
   | Linux | `/usr/OFX/Plugins` |

   Other folders: list them in `OFX_PLUGIN_PATH`, or use **Effect ▸ Load Plug-ins Through Helper…**.
4. **Start EffectCraft.** It starts the helper, which scans the plug-ins; large suites take a few
   seconds. The effects appear in **Effects & Presets** under their own categories.

Check it's working: `effectcraft-helper-ofx --list` prints every effect the helper finds, and
the control command `effect.helpers.list` shows whether EffectCraft started it.

## Using it

- Apply OFX effects like any other effect. Their ids are the OFX identifiers, lower-cased
  (projects made with the earlier in-process fork build keep working).
- **Effect ▸ Scan for Effect Helpers** starts the helper again if you installed it while
  EffectCraft was running.
- **Effect ▸ Load Plug-ins Through Helper…** loads a plug-in folder outside the standard
  locations.
- If something goes wrong, hover the ⚠ next to the effect name in Effect Controls. An effect
  that crashes the helper 3 times is turned off; `effect.helpers.scan {"retry": true}` turns it
  back on. To start EffectCraft without any helper, set `EFFECTCRAFT_NO_HELPERS=1`.
- Plug-in trials render a watermark until you activate them with the vendor's tools.

Tips for RSMB, Twixtor and Sapphire, how OFX parameters and contexts map onto EffectCraft, and
what isn't supported: **[docs/GUIDE.md](docs/GUIDE.md)**.

## Build it yourself

You need [Rust](https://rustup.rs/) 1.95 or newer (on Windows, the Visual Studio C++ build
tools).

```sh
git clone https://github.com/glumbis/ec-extension-ofx
cd ec-extension-ofx
cargo build --release
# → target/release/effectcraft-helper-ofx (.exe on Windows)
./target/release/effectcraft-helper-ofx --list
```

Release zips with checksums: `./package.sh` (or `pwsh package.ps1` on Windows) → `dist/`.

Tests, including an end-to-end run through the real helper with the bundled test plug-in:
[docs/GUIDE.md § Testing the host](docs/GUIDE.md#testing-the-host).

## How it's put together

| Path | What |
|---|---|
| `src/main.rs` | The helper program: loads OFX plug-ins and serves them with `effectcraft_helper_host::serve` |
| `ofx-host/` | `effectcraft-ofx`, the OpenFX 1.4 host (property, parameter, image effect, memory, multithread, message, progress, timeline suites; clips; the crash-on-load blocklist) |
| `test-plugin/` | `ec-test-ofx`, a tiny dependency-free OFX plug-in used by the tests |
| `tests/through_helper.rs` | End to end: EffectCraft's helper host ⇄ this helper ⇄ the test plug-in, including a killed helper |

The protocol is EffectCraft's
[external effect helper protocol](https://github.com/glumbis/effectcraft/blob/claude/openfx-external-helper-jdpn38/docs/external-helpers.md),
and the reasons for the design are in its
[ADR](https://github.com/glumbis/effectcraft/blob/claude/openfx-external-helper-jdpn38/docs/adr/0001-external-effect-helpers.md).
Any program that speaks it can be a helper; this one is written in Rust and reuses EffectCraft's
plug-in types.

## Related

- [EffectCraft](https://github.com/storytold/effectcraft): the official project.
- [glumbis/effectcraft](https://github.com/glumbis/effectcraft): the unofficial fork with an
  all-in-one Windows download (OpenFX built in, plus `.aep` import).
- [OpenFX](https://github.com/AcademySoftwareFoundation/openfx) · [docs](https://openfx.readthedocs.io).

## License

MIT OR Apache-2.0. See [ATTRIBUTION.md](ATTRIBUTION.md) for the OpenFX headers (BSD-3-Clause).
Built by [@glumbis](https://github.com/glumbis), mostly written with Claude Code.
