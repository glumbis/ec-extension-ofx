# Attribution

ec-extension-ofx is licensed MIT OR Apache-2.0 (see `LICENSE-MIT`, `LICENSE-APACHE`).

It builds on these crates from [EffectCraft](https://github.com/storytold/effectcraft)
(MIT OR Apache-2.0, © the EffectCraft contributors): `effectcraft-effects` and
`effectcraft-helper-host`.

And from this BSD-3-Clause project:

- `ofx-host/src/ffi.rs` (and the test plug-in's `test-plugin/src/ffi.rs`) transcribe the interface
  of the OpenFX API headers (`ofxCore.h`, `ofxProperty.h`, `ofxParam.h`, `ofxImageEffect.h` and
  the other suite headers): struct layouts, status codes and property, action and suite names.
  Copyright OpenFX and contributors to the OpenFX project (The Open Effects Association / Academy
  Software Foundation), https://github.com/AcademySoftwareFoundation/openfx, BSD-3-Clause. No host
  implementation code is copied.

OpenFX plug-ins themselves (Sapphire, RSMB, Twixtor, ...) are not included and are not unlocked
by this project; they are installed and licensed with their vendors' own tools.
