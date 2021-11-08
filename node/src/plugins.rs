use crate::{
    config::{self},
    prelude::*,
};
use neon::{prelude::*, result::Throw};
use std::str::FromStr;
use stencila::{
    config::Config,
    tokio::sync::MutexGuard,
};
use plugins::{self, Plugin, PluginInstallation, Plugins, PLUGINS};

/// Lock the global plugins store
pub fn lock(cx: &mut FunctionContext) -> NeonResult<MutexGuard<'static, Plugins>> {
    match PLUGINS.try_lock() {
        Ok(guard) => Ok(guard),
        Err(error) => cx.throw_error(format!(
            "When attempting to lock plugins: {}",
            error.to_string()
        )),
    }
}

/// Get plugin schema
pub fn schema(cx: FunctionContext) -> JsResult<JsString> {
    let schema = Plugin::schema();
    to_json_or_throw(cx, schema)
}

/// List plugins
pub fn list(mut cx: FunctionContext) -> JsResult<JsString> {
    let aliases = &config::lock(&mut cx)?.plugins.aliases;
    let plugins = &*lock(&mut cx)?;

    to_json(cx, plugins.list_plugins(aliases))
}

/// Install a plugin
pub fn install(mut cx: FunctionContext) -> JsResult<JsString> {
    let spec = &cx.argument::<JsString>(0)?.value(&mut cx);

    let config = &config::lock(&mut cx)?;
    let installs = &installations(&mut cx, 1, config)?;
    let aliases = &config.plugins.aliases;
    let plugins = &mut *lock(&mut cx)?;

    match RUNTIME.block_on(async { Plugin::install(spec, installs, aliases, plugins, None).await })
    {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Uninstall a plugin
pub fn uninstall(mut cx: FunctionContext) -> JsResult<JsString> {
    let alias = &cx.argument::<JsString>(0)?.value(&mut cx);
    let aliases = &config::lock(&mut cx)?.plugins.aliases;
    let plugins = &mut *lock(&mut cx)?;

    match Plugin::uninstall(alias, aliases, plugins) {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Upgrade a plugin
pub fn upgrade(mut cx: FunctionContext) -> JsResult<JsString> {
    let spec = &cx.argument::<JsString>(0)?.value(&mut cx);
    let config = &config::lock(&mut cx)?;
    let installs = &config.plugins.installations;
    let aliases = &config.plugins.aliases;
    let plugins = &mut *lock(&mut cx)?;

    match RUNTIME.block_on(async { Plugin::upgrade(spec, installs, aliases, plugins).await }) {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Refresh plugins
pub fn refresh(mut cx: FunctionContext) -> JsResult<JsString> {
    let arg = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?;
    let list = arg
        .iter()
        .map(|item| {
            item.to_string(&mut cx)
                .expect("Unable to convert to string")
                .value(&mut cx)
        })
        .collect();

    let config = &config::lock(&mut cx)?;
    let aliases = &config.plugins.aliases;
    let plugins = &mut *lock(&mut cx)?;

    match RUNTIME.block_on(async { Plugin::refresh_list(list, aliases, plugins).await }) {
        Ok(_) => to_json(cx, plugins.list_plugins(aliases)),
        Err(error) => cx.throw_error(error.to_string()),
    }
}

/// Get the `installations` argument, falling back to the array in `config.plugins.installations`
pub fn installations(
    cx: &mut FunctionContext,
    position: i32,
    config: &Config,
) -> Result<Vec<PluginInstallation>, Throw> {
    let arg = cx.argument::<JsArray>(position)?.to_vec(cx)?;
    if arg.is_empty() {
        Ok(config.plugins.installations.clone())
    } else {
        let mut installations = Vec::new();
        for value in arg {
            let str = value.to_string(cx)?.value(cx);
            let installation = match plugins::PluginInstallation::from_str(&str) {
                Ok(value) => value,
                Err(error) => return cx.throw_error(error.to_string()),
            };
            installations.push(installation)
        }
        Ok(installations)
    }
}
