use std::str::FromStr;

use common::{
    eyre::{self, Result},
    serde::Serialize,
};

pub const DOCKER_REGISTRY: &str = "registry.hub.docker.com";
pub const FLY_REGISTRY: &str = "registry.fly.io";

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(crate = "common::serde")]
pub struct ImageReference {
    /// The registry the image is on. Defaults to `registry.hub.docker.com`
    pub registry: String,

    /// The repository the image is in e.g. `ubuntu`, `library/hello-world`
    pub repository: String,

    /// An image tag e.g. `sha256:...`. Conflicts with `digest`.
    pub tag: Option<String>,

    /// An image digest e.g. `sha256:e07ee1baac5fae6a26f3...`. Conflicts with `tag`.
    pub digest: Option<String>,
}

impl ImageReference {
    /// Get the digest or tag for the reference, falling back to `latest`
    ///
    /// Use this when pulling a manifest to get the version that most closely
    /// matches that specified in the reference.
    pub fn digest_or_tag_or_latest(&self) -> String {
        match self.digest.as_ref().or_else(|| self.tag.as_ref()) {
            Some(reference) => reference.clone(),
            None => "latest".to_string(),
        }
    }

    /// Get the tag for the reference falling back to `latest`
    ///
    /// Use this when pushing a manifest for the image.
    pub fn tag_or_latest(&self) -> String {
        self.tag.clone().unwrap_or_else(|| "latest".to_string())
    }

    /// Convert reference to a string with `tag` or "latest" (i.e. not using any `digest`).
    pub fn to_string_tag_or_latest(&self) -> String {
        [
            &self.registry,
            "/",
            &self.repository,
            ":",
            self.tag.as_deref().unwrap_or("latest"),
        ]
        .concat()
    }
}

impl FromStr for ImageReference {
    type Err = eyre::Report;

    /// Parse a string into an [`ImageSpec`]
    ///
    /// Based on the implementation in https://github.com/HewlettPackard/dockerfile-parser-rs/
    fn from_str(str: &str) -> Result<ImageReference> {
        let parts: Vec<&str> = str.splitn(2, '/').collect();

        let first = parts[0];
        let (registry, rest) = if parts.len() == 2
            && (first == "localhost" || first.contains('.') || first.contains(':'))
        {
            (Some(parts[0]), parts[1])
        } else {
            (None, str)
        };

        let registry = if matches!(registry, None) || matches!(registry, Some("docker.io")) {
            DOCKER_REGISTRY.to_string()
        } else {
            registry
                .expect("Should be Some because of the match above")
                .to_string()
        };

        let (name, tag, hash) = if let Some(at_pos) = rest.find('@') {
            let (name, hash) = rest.split_at(at_pos);
            (name.to_string(), None, Some(hash[1..].to_string()))
        } else {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            let name = parts[0].to_string();
            let tag = parts.get(1).map(|str| str.to_string());
            (name, tag, None)
        };

        let name = if registry == DOCKER_REGISTRY && !name.contains('/') {
            ["library/", &name].concat()
        } else {
            name
        };

        Ok(ImageReference {
            registry,
            repository: name,
            tag,
            digest: hash,
        })
    }
}

impl ToString for ImageReference {
    fn to_string(&self) -> String {
        if let Some(digest) = &self.digest {
            [&self.registry, "/", &self.repository, "@", digest].concat()
        } else {
            self.to_string_tag_or_latest()
        }
    }
}
