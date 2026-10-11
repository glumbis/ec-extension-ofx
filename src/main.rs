//! `effectcraft-helper-ofx`: EffectCraft's OpenFX effect helper.
//!
//! Started by EffectCraft with `--effectcraft-helper`, it loads the OpenFX plug-ins in the
//! standard OFX folders (and `OFX_PLUGIN_PATH`) with `effectcraft-ofx`, then serves them over
//! the external effect helper protocol (`docs/external-helpers.md`). The plug-ins run in this
//! process: if one crashes, EffectCraft reports it and starts the helper again. Plug-ins that
//! crash the helper while loading are remembered (in `EFFECTCRAFT_HELPER_STATE_DIR`) and skipped
//! on the next start.
//!
//! Run by hand: `effectcraft-helper-ofx --list [path]` prints what it would offer.

use std::path::PathBuf;
use std::sync::Arc;

use effectcraft_effects::plugin::{EffectPlugin, plugin};
use effectcraft_helper_host::protocol::HELPER_ARG;
use effectcraft_helper_host::serve::{Provider, serve};

struct Ofx {
    /// Folders loaded at start-up (the standard OFX folders and `OFX_PLUGIN_PATH`).
    paths: Vec<PathBuf>,
}

/// Load `path` and return the effects it registered (or had registered before).
fn load(path: &std::path::Path) -> (Vec<Arc<dyn EffectPlugin>>, Vec<String>) {
    let report = effectcraft_ofx::load_path(path);
    let effects = report.loaded.iter().filter_map(|e| plugin(&e.id)).collect();
    (effects, report.errors)
}

impl Provider for Ofx {
    fn name(&self) -> String {
        "OpenFX".into()
    }
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
    fn loads(&self) -> Vec<String> {
        vec!["ofx".into(), "bundle".into()]
    }
    fn effects(&mut self) -> (Vec<Arc<dyn EffectPlugin>>, Vec<String>) {
        let (mut all, mut errors) = (vec![], vec![]);
        for p in &self.paths {
            if !p.exists() {
                continue;
            }
            let (e, errs) = load(p);
            all.extend(e);
            errors.extend(errs);
        }
        (all, errors)
    }
    fn load(&mut self, path: &str) -> Result<(Vec<Arc<dyn EffectPlugin>>, Vec<String>), String> {
        let (effects, errors) = load(std::path::Path::new(path));
        if effects.is_empty() && !errors.is_empty() {
            return Err(errors.join("; "));
        }
        Ok((effects, errors))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(dir) = std::env::var_os("EFFECTCRAFT_HELPER_STATE_DIR") {
        effectcraft_ofx::set_state_dir(PathBuf::from(dir));
    }
    let mut ofx = Ofx { paths: effectcraft_ofx::default_search_paths() };
    let code = if args.iter().any(|a| a == HELPER_ARG) {
        match serve(&mut ofx) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("effectcraft-helper-ofx: {e}");
                1
            }
        }
    } else if args.first().is_some_and(|a| a == "--list") {
        if let Some(p) = args.get(1) {
            ofx.paths = vec![PathBuf::from(p)];
        }
        let (effects, errors) = ofx.effects();
        for e in &effects {
            let m = e.manifest();
            println!("{}\t{}\t{}\t{} params", m.id, m.name, m.category, m.params.len());
        }
        for e in &errors {
            eprintln!("error: {e}");
        }
        println!("{} effect(s), {} error(s)", effects.len(), errors.len());
        0
    } else {
        eprintln!("effectcraft-helper-ofx {}: EffectCraft's OpenFX effect helper.", env!("CARGO_PKG_VERSION"));
        eprintln!("EffectCraft starts it with {HELPER_ARG}; put it in helpers/ next to EffectCraft.");
        eprintln!("`effectcraft-helper-ofx --list [path]` lists the plug-ins it finds.");
        2
    };
    // Stop the programs some plug-ins leave running (Sapphire's preset browser).
    effectcraft_ofx::shutdown();
    std::process::exit(code);
}
