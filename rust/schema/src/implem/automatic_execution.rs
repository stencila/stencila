use std::str::FromStr;

use crate::{prelude::*, AutomaticExecution};

impl AutomaticExecution {
    pub fn to_html_special(&self) -> String {
        self.to_string().to_lowercase()
    }
}

impl FromStr for AutomaticExecution {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        use AutomaticExecution::*;
        match string.to_lowercase().as_str() {
            "never" | "false" => Ok(Never),
            "needed" | "true" => Ok(Needed),
            "always" => Ok(Always),
            _ => bail!("Invalid string for `AutomaticExecution`: {string}"),
        }
    }
}
