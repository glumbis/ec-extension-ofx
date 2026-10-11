//! End to end: EffectCraft's helper host starts the real `effectcraft-helper-ofx`, loads the
//! test OpenFX bundle through it and renders the bundle's effects over the helper protocol.
//!
//! Set `EC_TEST_OFX_BUNDLE` to the built `ec-test-ofx.ofx.bundle` (see README "Testing the
//! host"). Without it every test prints a note and returns.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use effectcraft_effects::plugin::{EffectPlugin, PluginFrame, PluginHost, PluginImage, PluginParamKind, PluginParams, plugin};

type Px = [f32; 4];

const INVERT: &str = "org.effectcraft.test.invert";
const ECHO: &str = "org.effectcraft.test.echo";
const BORDER: &str = "org.effectcraft.test.border";

fn helper_exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_effectcraft-helper-ofx"))
}

/// Starts the helper and loads the bundle through it, once per test process (registration is
/// global). `None` when `EC_TEST_OFX_BUNDLE` is unset.
fn setup() -> Option<&'static serde_json::Value> {
    static LOADED: OnceLock<Option<serde_json::Value>> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let Some(bundle) = std::env::var_os("EC_TEST_OFX_BUNDLE") else {
                println!("note: EC_TEST_OFX_BUNDLE is not set; skipping the helper end-to-end tests");
                return None;
            };
            // Keep crash counts and the helper's own block list out of the user's state.
            effectcraft_helper_host::blocklist::set_state_dir(state_dir());
            let scan = effectcraft_helper_host::scan_paths(&[helper_exe()], false);
            assert_eq!(scan["helpers"].as_array().map_or(0, Vec::len), 1, "the helper did not start: {scan}");
            let bundle = bundle.to_string_lossy().into_owned();
            let loaded = effectcraft_helper_host::load_path(&bundle).unwrap_or_else(|e| panic!("loading {bundle} through the helper: {e}"));
            assert!(loaded["errors"].as_array().is_some_and(Vec::is_empty), "load errors: {loaded}");
            Some(loaded)
        })
        .as_ref()
}

fn state_dir() -> PathBuf {
    std::env::temp_dir().join(format!("ec-ofx-helper-e2e-{}", std::process::id()))
}

/// Tests share one helper process and one test kills it: run them one at a time.
fn serial() -> MutexGuard<'static, ()> {
    static M: Mutex<()> = Mutex::new(());
    M.lock().unwrap_or_else(PoisonError::into_inner)
}

fn effect(id: &str) -> Arc<dyn EffectPlugin> {
    let p = plugin(id).unwrap_or_else(|| panic!("{id} is not registered"));
    assert!(p.source().starts_with("helper:"), "{id} is not served by the helper: {}", p.source());
    p
}

/// Default values, with `set` overriding parameters by (suffix of) id.
fn defaults(p: &dyn EffectPlugin, set: &[(&str, &[f64])]) -> (Vec<f64>, Vec<String>) {
    let (mut flat, mut texts) = (vec![], vec![]);
    for q in &p.manifest().params {
        if let Some((_, v)) = set.iter().find(|(n, _)| q.id.to_lowercase().ends_with(&n.to_lowercase())) {
            flat.extend_from_slice(v);
            continue;
        }
        match &q.kind {
            PluginParamKind::Slider { default, .. } => flat.push(*default),
            PluginParamKind::Angle { default } => flat.push(*default),
            PluginParamKind::Checkbox { default } => flat.push(*default as u8 as f64),
            PluginParamKind::Popup { default, .. } => flat.push(*default as f64),
            PluginParamKind::Point { default } => flat.extend(default),
            PluginParamKind::Color { default } => flat.extend(default),
            PluginParamKind::Point3 { default } => flat.extend(default),
            PluginParamKind::Layer => flat.push(-1.0),
            PluginParamKind::Text { default } | PluginParamKind::Hidden { default } => texts.push(default.clone()),
        }
    }
    (flat, texts)
}

/// The host side: the layer at time `t` is a flat grey of `t / 4`, and every `input_at` time is
/// recorded.
struct Host {
    w: u32,
    h: u32,
    times: Mutex<Vec<f64>>,
}

fn grey(t: f64) -> Px {
    let v = (t / 4.0) as f32;
    [v, v, v, 1.0]
}

impl PluginHost for Host {
    fn input_at(&self, t: f64) -> Option<PluginImage> {
        self.times.lock().unwrap_or_else(PoisonError::into_inner).push(t);
        Some(PluginImage { width: self.w, height: self.h, pixels: vec![grey(t); (self.w * self.h) as usize], origin: [0.0; 2], scale: 1.0 })
    }
    fn layer_input(&self, _: &str, _: f64) -> Option<PluginImage> {
        None
    }
    fn params_at(&self, _: f64) -> Option<(Vec<f64>, Vec<String>)> {
        None
    }
    fn frame_rate(&self) -> f64 {
        10.0
    }
    fn time(&self) -> f64 {
        TIME
    }
    fn instance_key(&self) -> u64 {
        11
    }
}

const TIME: f64 = 1.0;

/// Renders a `w`×`h` layer the way the engine does: pad by `padding()` on every side, then
/// `render_v2`. Returns the padded width, the padding and the pixels.
fn run(id: &str, w: u32, h: u32, layer: &[Px], set: &[(&str, &[f64])], host: &Host) -> Result<(u32, u32, Vec<Px>), String> {
    let p = effect(id);
    let (flat, texts) = defaults(&*p, set);
    let params = PluginParams { manifest: p.manifest(), flat: &flat, texts: &texts };
    let pad = p.padding(&params, TIME, 1.0, [w as f64, h as f64], host.instance_key());
    let (fw, fh) = (w + 2 * pad, h + 2 * pad);
    let mut pixels = vec![[0.0; 4]; (fw * fh) as usize];
    for y in 0..h {
        for x in 0..w {
            pixels[((y + pad) * fw + x + pad) as usize] = layer[(y * w + x) as usize];
        }
    }
    let mut frame = PluginFrame { width: fw, height: fh, pixels: &mut pixels, scale: 1.0, origin: [pad as f64; 2], layer_size: [w as f64, h as f64] };
    p.render_v2(&mut frame, &params, TIME, host)?;
    Ok((fw, pad, pixels))
}

fn host(w: u32, h: u32) -> Host {
    Host { w, h, times: Mutex::new(vec![]) }
}

fn close(a: Px, b: Px) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
}

fn invert_works() -> Result<(), String> {
    let src = vec![[0.2, 0.4, 0.6, 1.0], [0.1, 0.2, 0.3, 0.5], [0.0; 4], [1.0; 4]];
    let (_, pad, out) = run(INVERT, 2, 2, &src, &[], &host(2, 2))?;
    assert_eq!(pad, 0);
    // Premultiplied: colour -> alpha - colour; alpha is kept.
    let want = [[0.8, 0.6, 0.4, 1.0], [0.4, 0.3, 0.2, 0.5], [0.0; 4], [0.0, 0.0, 0.0, 1.0]];
    for (g, w) in out.iter().zip(want) {
        assert!(close(*g, w), "got {g:?}, want {w:?}");
    }
    Ok(())
}

#[test]
fn effects_are_registered_through_the_helper() {
    let _s = serial();
    let Some(loaded) = setup() else { return };
    let ids: Vec<&str> = loaded["loaded"].as_array().into_iter().flatten().filter_map(|e| e["id"].as_str()).collect();
    for id in [INVERT, ECHO, BORDER, "org.effectcraft.test.mix", "org.effectcraft.test.allparams"] {
        assert!(ids.contains(&id), "{id} missing from {ids:?}");
        assert_eq!(effect(id).manifest().category, "EC Test");
    }
    let list = effectcraft_helper_host::list();
    assert_eq!(list["helpers"][0]["name"], "OpenFX", "{list}");
}

#[test]
fn invert_inverts() {
    let _s = serial();
    if setup().is_none() {
        return;
    }
    invert_works().expect("render");
    // amount = 0 leaves the image alone.
    let src = vec![[0.2, 0.4, 0.6, 1.0]; 4];
    let (.., out) = run(INVERT, 2, 2, &src, &[("amount", &[0.0])], &host(2, 2)).expect("render");
    assert!(out.iter().zip(&src).all(|(a, b)| close(*a, *b)), "{out:?}");
}

#[test]
fn echo_averages_the_previous_frame_from_the_host() {
    let _s = serial();
    if setup().is_none() {
        return;
    }
    let (w, h) = (4, 3);
    let h0 = host(w, h);
    let now = vec![grey(TIME); (w * h) as usize];
    let (.., out) = run(ECHO, w, h, &now, &[], &h0).expect("render");
    // One frame back at 10 fps: (grey(1.0) + grey(0.9)) / 2 = (0.25 + 0.225) / 2.
    let want = [0.2375, 0.2375, 0.2375, 1.0];
    for g in &out {
        assert!(close(*g, want), "got {g:?}, want {want:?}");
    }
    let times = h0.times.lock().unwrap_or_else(PoisonError::into_inner).clone();
    assert!(times.iter().any(|t| (t - (TIME - 0.1)).abs() < 1e-6), "input_at times: {times:?}");
}

#[test]
fn border_grows_the_frame() {
    let _s = serial();
    if setup().is_none() {
        return;
    }
    let (w, h) = (6, 4);
    let green = vec![[0.0, 1.0, 0.0, 1.0]; (w * h) as usize];
    let (fw, pad, out) = run(BORDER, w, h, &green, &[("size", &[3.0])], &host(w, h)).expect("render");
    assert!(pad >= 3, "padding() should grow with the region of definition: {pad}");
    let fh = h + 2 * pad;
    assert_eq!(out.len(), (fw * fh) as usize);
    let at = |x: u32, y: u32| out[(y * fw + x) as usize];
    for (x, y) in [(0, 0), (fw - 1, fh - 1), (fw - 1, 0), (0, fh - 1)] {
        assert!(close(at(x, y), [1.0, 0.0, 0.0, 1.0]), "corner ({x},{y}) = {:?}", at(x, y));
    }
    assert!(close(at(pad, pad), [0.0, 1.0, 0.0, 1.0]), "{:?}", at(pad, pad));
    assert!(close(at(pad + w - 1, pad + h - 1), [0.0, 1.0, 0.0, 1.0]));
}

/// The pids of this process's running `effectcraft-helper-ofx` children.
#[cfg(target_os = "linux")]
fn helper_pids() -> Vec<u32> {
    let me = std::process::id();
    let Ok(rd) = std::fs::read_dir("/proc") else { return vec![] };
    rd.flatten()
        .filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok())
        .filter(|pid| {
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else { return false };
            // `pid (comm) state ppid ...`: comm may hold spaces, so split after the last ')'.
            let mut rest = stat.rsplit_once(')').map_or("", |(_, r)| r).split_whitespace();
            let (state, ppid) = (rest.next(), rest.next().and_then(|p| p.parse::<u32>().ok()));
            let cmd = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
            ppid == Some(me) && state != Some("Z") && String::from_utf8_lossy(&cmd).contains("effectcraft-helper-ofx")
        })
        .collect()
}

#[test]
fn the_helper_restarts_after_it_is_killed_or_stopped() {
    let _s = serial();
    if setup().is_none() {
        return;
    }
    invert_works().expect("render before the kill");

    #[cfg(target_os = "linux")]
    {
        let pids = helper_pids();
        assert_eq!(pids.len(), 1, "expected one running helper, found {pids:?}");
        let status = std::process::Command::new("kill").arg("-9").arg(pids[0].to_string()).status().expect("run kill");
        assert!(status.success());
        // Wait for it to die (it stays a zombie until the host reaps it).
        let t0 = std::time::Instant::now();
        while helper_pids().contains(&pids[0]) && t0.elapsed() < std::time::Duration::from_secs(10) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // The padding request finds the process gone; the render starts the helper again,
        // which reloads the bundle, and succeeds.
        invert_works().expect("render after the kill");
        let now = helper_pids();
        assert!(now.len() == 1 && now[0] != pids[0], "a new helper should run: {now:?} (killed {})", pids[0]);
    }

    // A stopped helper starts again on the very next render, even one not preceded by a
    // padding request (which would start it first).
    effectcraft_helper_host::stop_all();
    assert_eq!(effectcraft_helper_host::list()["helpers"][0]["running"], false);
    let p = effect(INVERT);
    let (flat, texts) = defaults(&*p, &[]);
    let params = PluginParams { manifest: p.manifest(), flat: &flat, texts: &texts };
    let mut px = vec![[0.25, 0.5, 1.0, 1.0]; 4];
    let mut frame = PluginFrame { width: 2, height: 2, pixels: &mut px, scale: 1.0, origin: [0.0; 2], layer_size: [2.0; 2] };
    p.render_v2(&mut frame, &params, TIME, &host(2, 2)).expect("render after stop_all");
    assert!(px.iter().all(|g| close(*g, [0.75, 0.5, 0.0, 1.0])), "{px:?}");
    effectcraft_helper_host::stop_all();
    invert_works().expect("render with padding after stop_all");
    let list = effectcraft_helper_host::list();
    assert_eq!(list["helpers"][0]["running"], true, "{list}");
    // The reloaded bundle's temporal effect works in the new process too.
    let h0 = host(2, 2);
    let (.., out) = run(ECHO, 2, 2, &[grey(TIME); 4], &[], &h0).expect("echo after restart");
    assert!(out.iter().all(|g| close(*g, [0.2375, 0.2375, 0.2375, 1.0])), "{out:?}");
    // The kill was charged to Invert once: not enough to turn it off.
    assert!(!effectcraft_helper_host::blocklist::is_blocked(INVERT));
    let _ = std::fs::remove_dir_all(state_dir());
}
