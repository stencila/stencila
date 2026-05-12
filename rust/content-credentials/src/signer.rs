//! Signing identity configuration and local signing identity generation.

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use c2pa::{BoxedSigner, create_signer, crypto::raw_signature::SigningAlg};
use chrono::{Datelike, Duration, Utc};
use rcgen::{
    CertificateParams, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose, date_time_ymd,
};

use stencila_dirs::{DirType, get_app_dir};

use crate::error::{Error, Result};

/// Common name carried by Stencila's local signing identity. The literal
/// `(untrusted)` substring is intentional — it surfaces in the verifier's
/// signer line so readers can immediately tell this is a local self-signed
/// identity.
pub const LOCAL_SIGNING_IDENTITY_COMMON_NAME: &str = "Stencila Local Signing Identity (untrusted)";

const LOCAL_SIGNING_CERT_FILENAME: &str = "local-signing-cert.pem";
const LOCAL_SIGNING_KEY_FILENAME: &str = "local-signing-key.pem";

const ENV_CERT: &str = "STENCILA_CREDENTIALS_CERT";
const ENV_KEY: &str = "STENCILA_CREDENTIALS_KEY";
const ENV_TSA_URL: &str = "STENCILA_CREDENTIALS_TSA_URL";

/// Resolved signer configuration.
#[derive(Debug, Clone)]
pub struct CredentialSignerConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub alg: SigningAlg,
    pub tsa_url: Option<String>,
}

impl CredentialSignerConfig {
    /// Resolve a signer config from explicit overrides, environment, or the
    /// default local signing identity location.
    ///
    /// Precedence: explicit overrides → environment variables → local signing
    /// identity in the Stencila config directory → legacy dev cert in the
    /// config directory → [`Error::NoSignerConfigured`].
    ///
    /// # Errors
    ///
    /// Returns an error if only one explicit override or environment variable
    /// is supplied, a configured certificate or key path does not exist, the
    /// credentials directory cannot be resolved, or no signer is configured.
    pub fn resolve(cert_override: Option<PathBuf>, key_override: Option<PathBuf>) -> Result<Self> {
        Self::resolve_with_options(cert_override, key_override, None)
    }

    /// Resolve a signer config, optionally overriding the timestamp authority URL.
    ///
    /// The TSA URL is resolved from the explicit override first, then
    /// `STENCILA_CREDENTIALS_TSA_URL`, then left unset.
    ///
    /// # Errors
    ///
    /// Returns an error if only one explicit override or environment variable
    /// is supplied, a configured certificate or key path does not exist, the
    /// credentials directory cannot be resolved, or no signer is configured.
    pub fn resolve_with_options(
        cert_override: Option<PathBuf>,
        key_override: Option<PathBuf>,
        tsa_url_override: Option<String>,
    ) -> Result<Self> {
        let tsa_url = tsa_url_override.or_else(|| {
            env::var(ENV_TSA_URL)
                .ok()
                .filter(|url| !url.trim().is_empty())
        });

        match (cert_override, key_override) {
            (Some(cert), Some(key)) => return Self::from_paths(cert, key, tsa_url),
            (Some(_), None) | (None, Some(_)) => return Err(Error::SignerOverridesIncomplete),
            (None, None) => {}
        }

        if let (Ok(cert), Ok(key)) = (env::var(ENV_CERT), env::var(ENV_KEY)) {
            return Self::from_paths(PathBuf::from(cert), PathBuf::from(key), tsa_url);
        }
        if env::var_os(ENV_CERT).is_some() || env::var_os(ENV_KEY).is_some() {
            return Err(Error::SignerEnvIncomplete(ENV_CERT, ENV_KEY));
        }

        let dir = credentials_dir(false)?;
        let cert = dir.join(LOCAL_SIGNING_CERT_FILENAME);
        let key = dir.join(LOCAL_SIGNING_KEY_FILENAME);
        if cert.exists() && key.exists() {
            return Self::from_paths(cert, key, tsa_url);
        }

        Err(Error::NoSignerConfigured)
    }

    fn from_paths(cert_path: PathBuf, key_path: PathBuf, tsa_url: Option<String>) -> Result<Self> {
        if !cert_path.exists() {
            return Err(Error::CertNotFound(cert_path));
        }
        if !key_path.exists() {
            return Err(Error::KeyNotFound(key_path));
        }
        Ok(Self {
            cert_path,
            key_path,
            alg: SigningAlg::Es256,
            tsa_url,
        })
    }

    /// Build a c2pa signer from this configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured certificate or key cannot be read or
    /// parsed by the c2pa SDK.
    pub fn create_signer(&self) -> Result<BoxedSigner> {
        let signer = create_signer::from_files(
            &self.cert_path,
            &self.key_path,
            self.alg,
            self.tsa_url.clone(),
        )?;
        Ok(signer)
    }
}

/// The directory under the user's Stencila config root where local signing
/// identity material is stored.
///
/// # Errors
///
/// Returns an error if the Stencila config directory cannot be resolved or,
/// when `ensure` is true, the credentials directory cannot be created.
pub fn credentials_dir(ensure: bool) -> Result<PathBuf> {
    let config_dir =
        get_app_dir(DirType::Config, ensure).map_err(|err| Error::other(err.to_string()))?;
    let dir = config_dir.join("credentials");
    if ensure && !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Result of running [`init_local_signing_identity`].
#[derive(Debug, Clone)]
pub struct LocalSigningIdentityInit {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub created: bool,
    pub common_name: String,
}

/// Generate (or refresh) the local signing identity at
/// `<config>/credentials/local-signing-cert.pem` + `local-signing-key.pem`.
///
/// The certificate's common name carries `(untrusted)` so it surfaces in
/// verifier output. Pass `force = true` to overwrite an existing pair.
///
/// # Errors
///
/// Returns an error if the credentials directory cannot be created, certificate
/// generation fails, or the certificate or key cannot be written.
pub fn init_local_signing_identity(force: bool) -> Result<LocalSigningIdentityInit> {
    let dir = credentials_dir(true)?;
    let cert_path = dir.join(LOCAL_SIGNING_CERT_FILENAME);
    let key_path = dir.join(LOCAL_SIGNING_KEY_FILENAME);

    if !force && cert_path.exists() && key_path.exists() {
        return Ok(LocalSigningIdentityInit {
            cert_path,
            key_path,
            created: false,
            common_name: LOCAL_SIGNING_IDENTITY_COMMON_NAME.to_string(),
        });
    }

    let mut params =
        CertificateParams::new(vec!["stencila.local".to_string(), "localhost".to_string()])?;

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, LOCAL_SIGNING_IDENTITY_COMMON_NAME);
    dn.push(DnType::OrganizationName, "Stencila Local");
    params.distinguished_name = dn;

    // C2PA certificate profile requirements for end-entity signing certs.
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    // C2PA disallows the `anyExtendedKeyUsage` purpose. Only emailProtection
    // is included; it is one of the EKUs the default trust policy accepts.
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::EmailProtection];
    // C2PA also requires an Authority Key Identifier extension on the
    // signing certificate (for chain validation, even on a self-signed cert).
    params.use_authority_key_identifier_extension = true;

    let now = Utc::now();
    params.not_before = date_time_ymd(now.year(), 1, 1);
    let later = now + Duration::days(365);
    params.not_after = date_time_ymd(later.year(), 12, 31);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    write_secret(&cert_path, cert.pem().as_bytes())?;
    write_secret(&key_path, key_pair.serialize_pem().as_bytes())?;

    Ok(LocalSigningIdentityInit {
        cert_path,
        key_path,
        created: true,
        common_name: LOCAL_SIGNING_IDENTITY_COMMON_NAME.to_string(),
    })
}

fn write_secret(path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        if path.exists() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }

        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).mode(0o600);

        let file = options.open(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        file.set_len(0)?;
        file
    };

    #[cfg(not(unix))]
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(bytes)?;
    file.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures generated local signing identities visibly advertise their untrusted status.
    #[test]
    fn local_signing_identity_common_name_marked_untrusted() {
        assert!(LOCAL_SIGNING_IDENTITY_COMMON_NAME.contains("untrusted"));
    }

    /// Ensures local signing identity files no longer use development naming.
    #[test]
    fn local_signing_identity_filenames_are_user_facing() {
        assert_eq!(LOCAL_SIGNING_CERT_FILENAME, "local-signing-cert.pem");
        assert_eq!(LOCAL_SIGNING_KEY_FILENAME, "local-signing-key.pem");
    }

    /// Ensures explicit signer overrides require both certificate and key paths.
    #[test]
    fn resolve_requires_both_overrides() {
        let err =
            CredentialSignerConfig::resolve(Some(PathBuf::from("/nope")), None).expect_err("err");
        assert!(matches!(err, Error::SignerOverridesIncomplete));
    }

    /// Ensures signer resolution reports a missing certificate before using a missing key.
    #[test]
    fn resolve_missing_cert_file() {
        let err = CredentialSignerConfig::resolve(
            Some(PathBuf::from("/definitely/missing.pem")),
            Some(PathBuf::from("/definitely/missing.key")),
        )
        .expect_err("err");
        assert!(matches!(err, Error::CertNotFound(_)));
    }

    /// Ensures explicit TSA URLs are carried into resolved signer configuration.
    #[test]
    fn resolve_with_options_sets_tsa_url() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cert = tmp.path().join("cert.pem");
        let key = tmp.path().join("key.pem");
        fs::write(&cert, "cert")?;
        fs::write(&key, "key")?;

        let config = CredentialSignerConfig::resolve_with_options(
            Some(cert),
            Some(key),
            Some("https://tsa.example.test".to_string()),
        )?;

        assert_eq!(config.tsa_url.as_deref(), Some("https://tsa.example.test"));

        Ok(())
    }

    /// Ensures secret material is written with private permissions and repairs loose ones.
    #[cfg(unix)]
    #[test]
    fn write_secret_uses_private_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("secret.pem");

        write_secret(&path, b"secret")?;
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);

        fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
        write_secret(&path, b"new secret")?;
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);

        Ok(())
    }
}
