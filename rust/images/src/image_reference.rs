use std::str::FromStr;

use common::{
    eyre::{self, Result},
    serde::Serialize,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
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
        match self.digest.as_ref().or(self.tag.as_ref()) {
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

        let registry = registry.unwrap_or("docker.io").to_string();

        let (name, tag, hash) = if let Some(at_pos) = rest.find('@') {
            let (name, hash) = rest.split_at(at_pos);
            (name.to_string(), None, Some(hash[1..].to_string()))
        } else {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            let name = parts[0].to_string();
            let tag = parts.get(1).map(|str| str.to_string());
            (name, tag, None)
        };

        let name = if registry == "docker.io" && !name.contains('/') {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing image spec
    #[test]
    fn parse_image_ref() -> Result<()> {
        let ubuntu = ImageReference {
            registry: "docker.io".to_string(),
            repository: "library/ubuntu".to_string(),
            ..Default::default()
        };

        assert_eq!("ubuntu".parse::<ImageReference>()?, ubuntu);
        assert_eq!("docker.io/ubuntu".parse::<ImageReference>()?, ubuntu);
        assert_eq!("docker.io/ubuntu".parse::<ImageReference>()?, ubuntu);

        let ubuntu_2204 = ImageReference {
            registry: "docker.io".to_string(),
            repository: "library/ubuntu".to_string(),
            tag: Some("22.04".to_string()),
            ..Default::default()
        };

        assert_eq!("ubuntu:22.04".parse::<ImageReference>()?, ubuntu_2204);
        assert_eq!(
            "docker.io/ubuntu:22.04".parse::<ImageReference>()?,
            ubuntu_2204
        );
        assert_eq!(
            "docker.io/ubuntu:22.04".parse::<ImageReference>()?,
            ubuntu_2204
        );

        let ubuntu_digest = ImageReference {
            registry: "docker.io".to_string(),
            repository: "library/ubuntu".to_string(),
            digest: Some("sha256:abcdef".to_string()),
            ..Default::default()
        };

        assert_eq!(
            "ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );
        assert_eq!(
            "docker.io/ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );
        assert_eq!(
            "docker.io/ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );

        Ok(())
    }
}
