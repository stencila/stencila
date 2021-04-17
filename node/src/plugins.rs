use crate::{
    config::{self},
    prelude::*,
};
use neon::{prelude::*, result::Throw};
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use stencila::{
    config::Config,
    once_cell::sync::Lazy,
    plugins::{self, Installation, Plugin, Plugins},
};

/// A global plugins store
///
/// The plugins store needs to be read on startup and then passed to various
/// functions in other modules on each invocation (for delegation to each plugin).
/// As for config, we want to avoid exposing that implementation detail in these bindings
/// so have this global mutable plugins store that gets loaded when the module is loaded,
/// updated in the functions below and then passed on to other functions
pub static PLUGINS: Lazy<Mutex<Plugins>> =
    Lazy::new(|| Mutex::new(Plugins::load().expect("Unable to load plugins")));

/// Obtain the plugins store
pub fn obtain(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Plugins>> {
    match PLUGINS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting on obtain plugins: {}",
            error.to_string()
        )),
    }
}

/// List plugins
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let aliases = &config::obtain(&mut cx)?.plugins.aliases;
    let plugins = &*obtain(&mut cx)?;

    to_json(cx, plugins.list_plugins(aliases))
}

/// Install a plugin
pub fn install(mut cx: FunctionContext) -> JsResult<JsString> {
    let spec = &cx.argument::<JsString>(0)?.value(&mut cx);

    let config = &config::obtain(&mut cx)?;
    let installs = &installations(&mut cx, 1, &config)?;
    let aliases = &config.plugins.aliases;
    let plugins = &mut *obtain(&mut cx)?;

    match runtime(&mut cx)?
        .block_on(async { Plugin::install(spec, installs, aliases, plugins, None).await })
    {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Uninstall a plugin
pub fn uninstall(mut cx: FunctionContext) -> JsResult<JsString> {
    let alias = &cx.argument::<JsString>(0)?.value(&mut cx);
    let aliases = &config::obtain(&mut cx)?.plugins.aliases;
    let plugins = &mut *obtain(&mut cx)?;

    match Plugin::uninstall(alias, aliases, plugins) {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Upgrade a plugin
pub fn upgrade(mut cx: FunctionContext) -> JsResult<JsString> {
    let spec = &cx.argument::<JsString>(0)?.value(&mut cx);
    let config = &config::obtain(&mut cx)?;
    let installs = &config.plugins.installations;
    let aliases = &config.plugins.aliases;
    let plugins = &mut *obtain(&mut cx)?;

    match runtime(&mut cx)?
        .block_on(async { Plugin::upgrade(spec, installs, aliases, plugins).await })
    {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Get the `installations` argument, falling back to the array in `config.plugins.installations`
pub fn installations(
    cx: &mut FunctionContext,
    position: i32,
    config: &Config,
) -> Result<Vec<Installation>, Throw> {
    let arg = cx.argument::<JsArray>(position)?.to_vec(cx)?;
    if arg.is_empty() {
        Ok(config.plugins.installations.clone())
    } else {
        let mut installations = Vec::new();
        for value in arg {
            let str = value.to_string(cx)?.value(cx);
            let installation = match plugins::Installation::from_str(&str) {
                Ok(value) => value,
                Err(error) => return cx.throw_error(error.to_string()),
            };
            installations.push(installation)
        }
        Ok(installations)
    }
}
