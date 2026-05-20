//! Signing identity configuration and local signing identity generation.

use std::{
    env, fmt, fs,
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

/// Common name carried by Stencila's local signing identity.
///
/// Verifiers determine and display trust status separately from the signer
/// name, so the certificate subject describes the generated signing identity.
pub const LOCAL_SIGNING_IDENTITY_COMMON_NAME: &str = "Stencila local signing identity";

/// Organization name carried by Stencila's local signing identity.
///
// Use an explicitly local subject organization because C2PA tooling surfaces it
// as the signature issuer.
const LOCAL_SIGNING_IDENTITY_ORG_NAME: &str = "Local self-signed identity";

const LOCAL_SIGNING_CERT_FILENAME: &str = "local-signing-cert.pem";
const LOCAL_SIGNING_KEY_FILENAME: &str = "local-signing-key.pem";

const ENV_CERT: &str = "STENCILA_CREDENTIALS_CERT";
const ENV_KEY: &str = "STENCILA_CREDENTIALS_KEY";
const ENV_TSA_URL: &str = "STENCILA_CREDENTIALS_TSA_URL";
const ENV_CLOUD_URL: &str = "STENCILA_C2PA_URL";
const DEFAULT_CLOUD_URL: &str = "https://c2pa.stencila.cloud/v1";

/// The signing backend selected for a Content Credentials operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSigningMode {
    /// Sign in the current process with locally available key material.
    Local,

    /// Sign through the Stencila Cloud signing service.
    Cloud,
}

impl CredentialSigningMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
        }
    }
}

/// Signing material available to a local signing process.
#[derive(Clone)]
pub enum CredentialSignerMaterial {
    /// Certificate chain and private key stored as files.
    Files {
        cert_path: PathBuf,
        key_path: PathBuf,
    },

    /// Certificate chain and private key supplied in memory.
    ///
    /// This is intended for the Cloud signing service, where secret material is
    /// injected at runtime and should not be written into the container image.
    Pem { cert_pem: Vec<u8>, key_pem: Vec<u8> },
}

// Avoid deriving `Debug`: PEM material can include private key bytes supplied
// by Cloud signing runtimes, and those must not appear in logs or test output.
impl fmt::Debug for CredentialSignerMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Files {
                cert_path,
                key_path,
            } => f
                .debug_struct("Files")
                .field("cert_path", cert_path)
                .field("key_path", key_path)
                .finish(),
            Self::Pem { cert_pem, key_pem } => f
                .debug_struct("Pem")
                .field("cert_pem", &format!("<redacted: {} bytes>", cert_pem.len()))
                .field("key_pem", &format!("<redacted: {} bytes>", key_pem.len()))
                .finish(),
        }
    }
}

/// Resolved local signer configuration.
#[derive(Debug, Clone)]
pub struct CredentialSignerConfig {
    pub material: CredentialSignerMaterial,
    pub alg: SigningAlg,
    pub tsa_url: Option<String>,
}

impl CredentialSignerConfig {
    /// Resolve a signer config from explicit overrides, environment, or the
    /// default local signing identity location.
    ///
    /// Precedence: explicit overrides → environment variables → local signing
    /// identity in the Stencila config directory → [`Error::NoSignerConfigured`].
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
            material: CredentialSignerMaterial::Files {
                cert_path,
                key_path,
            },
            alg: SigningAlg::Es256,
            tsa_url,
        })
    }

    /// Build a local signer config from in-memory PEM certificate and key
    /// material.
    ///
    /// This avoids requiring Cloud signing containers to materialize injected
    /// secrets on disk before creating a C2PA signer.
    #[must_use]
    pub fn from_pem(
        cert_pem: impl Into<Vec<u8>>,
        key_pem: impl Into<Vec<u8>>,
        tsa_url: Option<String>,
    ) -> Self {
        Self {
            material: CredentialSignerMaterial::Pem {
                cert_pem: cert_pem.into(),
                key_pem: key_pem.into(),
            },
            alg: SigningAlg::Es256,
            tsa_url,
        }
    }

    /// Build a c2pa signer from this configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured certificate or key cannot be read or
    /// parsed by the c2pa SDK.
    pub fn create_signer(&self) -> Result<BoxedSigner> {
        let signer = match &self.material {
            CredentialSignerMaterial::Files {
                cert_path,
                key_path,
            } => create_signer::from_files(cert_path, key_path, self.alg, self.tsa_url.clone())?,
            CredentialSignerMaterial::Pem { cert_pem, key_pem } => {
                create_signer::from_keys(cert_pem, key_pem, self.alg, self.tsa_url.clone())?
            }
        };
        Ok(signer)
    }
}

/// Configuration for Stencila Cloud signing.
#[derive(Clone, PartialEq, Eq)]
pub struct CredentialCloudSigningConfig {
    /// Base URL for the Stencila C2PA service.
    ///
    /// The signing endpoint is resolved by appending `/sign`. The default is
    /// `https://c2pa.stencila.cloud/v1`, which hosts both Stencila's Cloud
    /// signing extension and the C2PA Soft Binding Resolution API routes.
    pub base_url: String,

    /// Optional bearer API key for authenticated Cloud signing.
    pub token: Option<String>,

    /// Whether the Cloud service should also store the manifest and register a
    /// soft binding for the signed asset.
    ///
    /// This stays false until callers expose a separate CLI/config option for
    /// the sign + store + bind workflow.
    pub register_soft_binding: bool,
}

impl CredentialCloudSigningConfig {
    /// Resolve Cloud signing configuration from the environment and keyring.
    #[must_use]
    pub fn resolve() -> Self {
        let base_url = env::var(ENV_CLOUD_URL).unwrap_or_else(|_| DEFAULT_CLOUD_URL.to_string());
        let token = stencila_secrets::env_or_get(stencila_secrets::STENCILA_API_KEY)
            .or_else(|_| stencila_secrets::env_or_get(stencila_secrets::STENCILA_API_TOKEN))
            .ok()
            .filter(|token| !token.trim().is_empty());

        Self {
            base_url,
            token,
            register_soft_binding: false,
        }
    }

    /// Use a custom C2PA service base URL.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Use a bearer API key for Cloud signing.
    #[must_use]
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Request combined signing, manifest storage, and soft binding
    /// registration.
    #[must_use]
    pub fn with_register_soft_binding(mut self, register: bool) -> Self {
        self.register_soft_binding = register;
        self
    }

    /// Whether the config has enough authentication material to try Cloud signing.
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.token
            .as_deref()
            .is_some_and(|token| !token.trim().is_empty())
    }

    /// Require Cloud signing authentication.
    ///
    /// # Errors
    ///
    /// Returns an error if no bearer token is configured.
    pub(crate) fn require_authenticated(&self) -> Result<()> {
        if self.is_authenticated() {
            Ok(())
        } else {
            Err(Error::CloudSigningUnauthenticated)
        }
    }
}

impl Default for CredentialCloudSigningConfig {
    fn default() -> Self {
        Self::resolve()
    }
}

impl fmt::Debug for CredentialCloudSigningConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialCloudSigningConfig")
            .field("base_url", &self.base_url)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("register_soft_binding", &self.register_soft_binding)
            .finish()
    }
}

/// Signing configuration for a Content Credentials producer.
#[derive(Debug, Clone)]
pub enum CredentialSigningConfig {
    /// Sign in-process with local key material.
    Local(CredentialSignerConfig),

    /// Sign with Stencila Cloud.
    Cloud(CredentialCloudSigningConfig),

    /// Prefer Cloud signing and fall back to local signing when Cloud is not available.
    Auto {
        cloud: CredentialCloudSigningConfig,
        local: Option<CredentialSignerConfig>,
    },
}

impl CredentialSigningConfig {
    /// Resolve the default local signing configuration.
    ///
    /// This preserves the current CLI/export behavior while making the backend
    /// choice explicit at producer boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if the default local signing identity cannot be
    /// resolved.
    pub fn resolve_local() -> Result<Self> {
        Ok(Self::Local(CredentialSignerConfig::resolve(None, None)?))
    }

    /// Resolve automatic signing configuration.
    ///
    /// Cloud signing is selected when a Stencila Cloud token is available.
    /// Otherwise this falls back to the default local signing identity.
    ///
    /// # Errors
    ///
    /// Returns an error if no authenticated Cloud config is available and no
    /// local signing identity can be resolved.
    pub fn resolve_auto() -> Result<Self> {
        Self::resolve_auto_with_cloud_config(CredentialCloudSigningConfig::resolve())
    }

    /// Resolve automatic signing configuration with an explicit Cloud config.
    ///
    /// This is useful for callers and tests that construct Cloud configuration
    /// directly instead of reading environment variables and the keyring.
    ///
    /// # Errors
    ///
    /// Returns an error if the Cloud config is not authenticated and no local
    /// signing identity can be resolved.
    pub fn resolve_auto_with_cloud_config(cloud: CredentialCloudSigningConfig) -> Result<Self> {
        if cloud.is_authenticated() {
            Ok(Self::Auto {
                cloud,
                local: CredentialSignerConfig::resolve(None, None).ok(),
            })
        } else {
            Self::resolve_local()
        }
    }

    /// Resolve the default Cloud signing configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if no Stencila Cloud API key is available.
    pub fn resolve_cloud() -> Result<Self> {
        let cloud = CredentialCloudSigningConfig::resolve();
        cloud.require_authenticated()?;
        Ok(Self::Cloud(cloud))
    }

    #[must_use]
    pub const fn mode(&self) -> CredentialSigningMode {
        match self {
            Self::Local(_) => CredentialSigningMode::Local,
            Self::Cloud(_) | Self::Auto { .. } => CredentialSigningMode::Cloud,
        }
    }
}

impl From<CredentialSignerConfig> for CredentialSigningConfig {
    fn from(config: CredentialSignerConfig) -> Self {
        Self::Local(config)
    }
}

impl From<CredentialCloudSigningConfig> for CredentialSigningConfig {
    fn from(config: CredentialCloudSigningConfig) -> Self {
        Self::Cloud(config)
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
/// Pass `force = true` to overwrite an existing pair.
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
    dn.push(DnType::OrganizationName, LOCAL_SIGNING_IDENTITY_ORG_NAME);
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

    /// Ensures generated local signing identities describe the signer, not the generator.
    #[test]
    fn local_signing_identity_common_name_describes_identity() {
        assert_eq!(
            LOCAL_SIGNING_IDENTITY_COMMON_NAME,
            "Stencila local signing identity"
        );
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

    /// Ensures in-memory PEM material can be configured without file paths.
    #[test]
    fn from_pem_uses_in_memory_material() {
        let config = CredentialSignerConfig::from_pem(
            b"cert".to_vec(),
            b"key".to_vec(),
            Some("https://tsa.example.test".to_string()),
        );

        assert!(matches!(
            config.material,
            CredentialSignerMaterial::Pem { .. }
        ));
        assert_eq!(config.tsa_url.as_deref(), Some("https://tsa.example.test"));
    }

    /// Ensures in-memory PEM material is redacted in debug output.
    #[test]
    fn from_pem_redacts_debug_output() {
        let cert = "test-cert-pem";
        let key = "test-private-key-pem";
        let config = CredentialSignerConfig::from_pem(
            cert.as_bytes().to_vec(),
            key.as_bytes().to_vec(),
            Some("https://tsa.example.test".to_string()),
        );

        let material_debug = format!("{:?}", config.material);
        let config_debug = format!("{config:?}");

        assert!(!material_debug.contains(cert));
        assert!(!material_debug.contains(key));
        assert!(!config_debug.contains(cert));
        assert!(!config_debug.contains(key));
        assert!(config_debug.contains("<redacted:"));
    }

    /// Ensures Cloud signing config does not expose bearer tokens in debug output.
    #[test]
    fn cloud_config_redacts_debug_output() {
        let token = "secret-cloud-token";
        let config = CredentialCloudSigningConfig {
            base_url: "https://c2pa.example.test/v1".to_string(),
            token: Some(token.to_string()),
            register_soft_binding: false,
        };

        let debug = format!("{config:?}");

        assert!(!debug.contains(token));
        assert!(debug.contains("<redacted>"));
    }

    /// Ensures explicit Cloud signing fails before attempting an anonymous request.
    #[test]
    fn cloud_config_requires_authentication() {
        let config = CredentialCloudSigningConfig {
            base_url: "https://c2pa.example.test/v1".to_string(),
            token: None,
            register_soft_binding: false,
        };

        let error = config
            .require_authenticated()
            .expect_err("unauthenticated Cloud config should fail");

        assert!(matches!(error, Error::CloudSigningUnauthenticated));
    }

    /// Ensures auto signing selects Cloud when Cloud auth is available.
    #[test]
    fn resolve_auto_selects_authenticated_cloud() -> Result<()> {
        let token = "secret-cloud-token";
        let config = CredentialCloudSigningConfig {
            base_url: "https://c2pa.example.test/v1".to_string(),
            token: Some(token.to_string()),
            register_soft_binding: false,
        };

        let signing = CredentialSigningConfig::resolve_auto_with_cloud_config(config)?;

        assert!(matches!(signing, CredentialSigningConfig::Auto { .. }));
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
