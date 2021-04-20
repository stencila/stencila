use neon::prelude::*;

mod config;
mod plugins;
mod prelude;
mod subscriptions;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("pluginsList", plugins::list)?;
    cx.export_function("pluginsInstall", plugins::install)?;
    cx.export_function("pluginsUninstall", plugins::uninstall)?;
    cx.export_function("pluginsUpgrade", plugins::upgrade)?;
    cx.export_function("pluginsRefresh", plugins::refresh)?;

    cx.export_function("configRead", config::read)?;
    cx.export_function("configWrite", config::write)?;
    cx.export_function("configValidate", config::validate)?;
    cx.export_function("configSet", config::set)?;
    cx.export_function("configReset", config::reset)?;

    cx.export_function("subscribe", subscriptions::subscribe)?;
    cx.export_function("unsubscribe", subscriptions::unsubscribe)?;
    cx.export_function("publish", subscriptions::publish)?;

    Ok(())
}
