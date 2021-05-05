use neon::prelude::*;

mod config;
mod logging;
mod plugins;
mod prelude;
mod pubsub;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("pluginsSchema", plugins::schema)?;
    cx.export_function("pluginsList", plugins::list)?;
    cx.export_function("pluginsInstall", plugins::install)?;
    cx.export_function("pluginsUninstall", plugins::uninstall)?;
    cx.export_function("pluginsUpgrade", plugins::upgrade)?;
    cx.export_function("pluginsRefresh", plugins::refresh)?;

    cx.export_function("configSchema", config::schema)?;
    cx.export_function("configRead", config::read)?;
    cx.export_function("configWrite", config::write)?;
    cx.export_function("configValidate", config::validate)?;
    cx.export_function("configSet", config::set)?;
    cx.export_function("configReset", config::reset)?;

    cx.export_function("pubsubInit", pubsub::init)?;
    cx.export_function("pubsubSubscribe", pubsub::subscribe)?;
    cx.export_function("pubsubUnsubscribe", pubsub::unsubscribe)?;
    cx.export_function("pubsubPublish", pubsub::publish)?;

    cx.export_function("loggingInit", logging::init)?;
    cx.export_function("loggingTest", logging::test)?;

    Ok(())
}
