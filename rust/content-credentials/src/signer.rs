//! Signing identity configuration and dev-cert generation.

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

/// Common name carried by Stencila's dev cert. The literal `(untrusted)`
/// substring is intentional — it surfaces in the verifier's signer line so
/// readers can immediately tell this is a local development identity.
pub const DEV_CERT_COMMON_NAME: &str = "Local Stencila Dev (untrusted)";

const DEV_CERT_FILENAME: &str = "dev-cert.pem";
const DEV_KEY_FILENAME: &str = "dev-key.pem";

const ENV_CERT: &str = "STENCILA_CREDENTIALS_CERT";
const ENV_KEY: &str = "STENCILA_CREDENTIALS_KEY";

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
    /// default dev-cert location.
    ///
    /// Precedence: explicit overrides → environment variables → dev cert in
    /// the Stencila config directory → [`Error::NoSignerConfigured`].
    ///
    /// # Errors
    ///
    /// Returns an error if only one explicit override or environment variable
    /// is supplied, a configured certificate or key path does not exist, the
    /// credentials directory cannot be resolved, or no signer is configured.
    pub fn resolve(cert_override: Option<PathBuf>, key_override: Option<PathBuf>) -> Result<Self> {
        match (cert_override, key_override) {
            (Some(cert), Some(key)) => return Self::from_paths(cert, key),
            (Some(_), None) | (None, Some(_)) => return Err(Error::SignerOverridesIncomplete),
            (None, None) => {}
        }

        if let (Ok(cert), Ok(key)) = (env::var(ENV_CERT), env::var(ENV_KEY)) {
            return Self::from_paths(PathBuf::from(cert), PathBuf::from(key));
        }
        if env::var_os(ENV_CERT).is_some() || env::var_os(ENV_KEY).is_some() {
            return Err(Error::SignerEnvIncomplete(ENV_CERT, ENV_KEY));
        }

        let dir = credentials_dir(false)?;
        let cert = dir.join(DEV_CERT_FILENAME);
        let key = dir.join(DEV_KEY_FILENAME);
        if cert.exists() && key.exists() {
            return Self::from_paths(cert, key);
        }

        Err(Error::NoSignerConfigured)
    }

    fn from_paths(cert_path: PathBuf, key_path: PathBuf) -> Result<Self> {
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
            tsa_url: None,
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

/// The directory under the user's Stencila config root where dev signing
/// material is stored.
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

/// Result of running [`init_dev_cert`].
#[derive(Debug, Clone)]
pub struct DevCertInit {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub created: bool,
    pub common_name: String,
}

/// Generate (or refresh) the local dev signing identity at
/// `<config>/credentials/dev-cert.pem` + `dev-key.pem`.
///
/// The certificate's common name carries `(untrusted)` so it surfaces in
/// verifier output. Pass `force = true` to overwrite an existing pair.
///
/// # Errors
///
/// Returns an error if the credentials directory cannot be created, certificate
/// generation fails, or the certificate or key cannot be written.
pub fn init_dev_cert(force: bool) -> Result<DevCertInit> {
    let dir = credentials_dir(true)?;
    let cert_path = dir.join(DEV_CERT_FILENAME);
    let key_path = dir.join(DEV_KEY_FILENAME);

    if !force && cert_path.exists() && key_path.exists() {
        return Ok(DevCertInit {
            cert_path,
            key_path,
            created: false,
            common_name: DEV_CERT_COMMON_NAME.to_string(),
        });
    }

    let mut params =
        CertificateParams::new(vec!["stencila.local".to_string(), "localhost".to_string()])?;

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, DEV_CERT_COMMON_NAME);
    dn.push(DnType::OrganizationName, "Stencila Local Dev");
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

    Ok(DevCertInit {
        cert_path,
        key_path,
        created: true,
        common_name: DEV_CERT_COMMON_NAME.to_string(),
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

    /// Ensures generated development certificates visibly advertise their untrusted status.
    #[test]
    fn dev_cert_common_name_marked_untrusted() {
        assert!(DEV_CERT_COMMON_NAME.contains("untrusted"));
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
