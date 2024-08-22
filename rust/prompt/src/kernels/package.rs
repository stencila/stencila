use crate::prelude::*;

use schema::StringOrNumber;

/// A package available within a kernel instance
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Package {
    /// The name of the package e.g. pandas
    #[qjs(get, enumerable)]
    name: String,

    /// The version of the package runtime e.g. 1.2.3
    #[qjs(get, enumerable)]
    version: Option<String>,
}

impl Package {
    #[cfg(test)]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
        }
    }
}

impl From<schema::SoftwareSourceCode> for Package {
    fn from(sw: schema::SoftwareSourceCode) -> Self {
        Self {
            name: sw.name,
            version: sw.version.map(|version| match version {
                StringOrNumber::String(string) => string,
                StringOrNumber::Number(num) => num.to_string(),
            }),
        }
    }
}

#[rquickjs::methods]
impl Package {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;

        obj.set("name", self.name.clone())?;
        obj.set("version", self.version.clone())?;

        Ok(obj)
    }
}
