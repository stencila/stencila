use neon::prelude::*;

mod config;
mod prelude;

register_module!(mut cx, {
    cx.export_function("configRead", config::read)?;
    cx.export_function("configWrite", config::write)?;
    cx.export_function("configValidate", config::validate)?;
    cx.export_function("configSet", config::set)?;
    cx.export_function("configReset", config::reset)?;
    Ok(())
});
