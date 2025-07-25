use crate::{
    environments::{Apt, Devbox},
    tool::{Tool, ToolType},
};

pub struct Git;

impl Tool for Git {
    fn name(&self) -> &'static str {
        "git"
    }

    fn url(&self) -> &'static str {
        "https://git-scm.com/"
    }

    fn description(&self) -> &'static str {
        "Distributed version control system"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Collaboration
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox), Box::new(Apt)]
    }
}
