use crate::command::{BuildCommand, Command};
use crate::config::{ConfigError, TEdgeConfig, DEVICE_CERT_PATH, DEVICE_KEY_PATH};
use crate::utils::paths;
use crate::utils::paths::PathsError;
use chrono::offset::Utc;
use chrono::Duration;
use rcgen::Certificate;
use rcgen::CertificateParams;
use rcgen::RcgenError;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum TEdgeCertOpt {
    /// Create a self-signed device certificate
    Create {
        /// The device identifier to be used as the common name for the certificate
        #[structopt(long = "device-id")]
        id: String,
    },

    /// Show the device certificate, if any
    Show,

    /// Remove the device certificate
    Remove,
}

/// Create a self-signed device certificate
pub struct CreateCertCmd {
    /// The device identifier
    id: String,

    /// The path where the device certificate will be stored
    cert_path: String,

    /// The path where the device private key will be stored
    key_path: String,
}

/// Show the device certificate, if any
pub struct ShowCertCmd {
    /// The path where the device certificate will be stored
    cert_path: String,
}

/// Remove the device certificate
pub struct RemoveCertCmd {
    /// The path of the certificate to be removed
    cert_path: String,

    /// The path of the private key to be removed
    key_path: String,
}

#[derive(thiserror::Error, Debug)]
pub enum CertError {
    #[error(r#"The string '{name:?}' contains characters which cannot be used in a name"#)]
    InvalidCharacter { name: String },

    #[error(r#"The empty string cannot be used as a name"#)]
    EmptyName,

    #[error(
        r#"The string '{name:?}' is more than {} characters long and cannot be used as a name"#,
        MAX_CN_SIZE
    )]
    TooLongName { name: String },

    #[error(
        r#"A certificate already exists and would be overwritten.
        Existing file: {path:?}
        Run `tegde cert remove` first to generate a new certificate.
    "#
    )]
    CertificateAlreadyExists { path: String },

    #[error(
        r#"No certificate has been attached to that device.
        Missing file: {path:?}
        Run `tegde cert create` to generate a new certificate.
    "#
    )]
    CertificateNotFound { path: String },

    #[error(
        r#"No private key has been attached to that device.
        Missing file: {path:?}
        Run `tegde cert create` to generate a new key and certificate.
    "#
    )]
    KeyNotFound { path: String },

    #[error(
        r#"A private key already exists and would be overwritten.
        Existing file: {path:?}
        Run `tegde cert remove` first to generate a new certificate and private key.
    "#
    )]
    KeyAlreadyExists { path: String },

    #[error(transparent)]
    ConfigError(#[from] ConfigError),

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("Invalid device.cert.path path: {0}")]
    CertPathError(PathsError),

    #[error("Invalid device.key.path path: {0}")]
    KeyPathError(PathsError),

    #[error("Cryptography related error")]
    CryptographyError(#[from] RcgenError),

    #[error("PEM file format error")]
    PemError(#[from] x509_parser::error::PEMError),

    #[error("X509 file format error: {0}")]
    X509Error(String), // One cannot use x509_parser::error::X509Error unless one use `nom`.
}

impl CertError {
    /// Improve the error message in case the error in a IO error on the certificate file.
    fn cert_context(self, path: &str) -> CertError {
        match self {
            CertError::IoError(ref err) => match err.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    CertError::CertificateAlreadyExists { path: path.into() }
                }
                std::io::ErrorKind::NotFound => {
                    CertError::CertificateNotFound { path: path.into() }
                }
                _ => self,
            },
            _ => self,
        }
    }

    /// Improve the error message in case the error in a IO error on the private key file.
    fn key_context(self, path: &str) -> CertError {
        match self {
            CertError::IoError(ref err) => match err.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    CertError::KeyAlreadyExists { path: path.into() }
                }
                std::io::ErrorKind::NotFound => CertError::KeyNotFound { path: path.into() },
                _ => self,
            },
            _ => self,
        }
    }
}

impl BuildCommand for TEdgeCertOpt {
    fn build_command(self, config: TEdgeConfig) -> Result<Box<dyn Command>, ConfigError> {
        let cmd =
            match self {
                TEdgeCertOpt::Create { id } => {
                    let cmd = CreateCertCmd {
                        id,
                        cert_path: config.device.cert_path.ok_or_else(|| {
                            ConfigError::ConfigNotSet {
                                key: String::from(DEVICE_CERT_PATH),
                            }
                        })?,
                        key_path: config.device.key_path.ok_or_else(|| {
                            ConfigError::ConfigNotSet {
                                key: String::from(DEVICE_KEY_PATH),
                            }
                        })?,
                    };
                    cmd.into_boxed()
                }

                TEdgeCertOpt::Show => {
                    let cmd = ShowCertCmd {
                        cert_path: config.device.cert_path.ok_or_else(|| {
                            ConfigError::ConfigNotSet {
                                key: String::from(DEVICE_CERT_PATH),
                            }
                        })?,
                    };
                    cmd.into_boxed()
                }

                TEdgeCertOpt::Remove => {
                    let cmd = RemoveCertCmd {
                        cert_path: config.device.cert_path.ok_or_else(|| {
                            ConfigError::ConfigNotSet {
                                key: String::from(DEVICE_CERT_PATH),
                            }
                        })?,
                        key_path: config.device.key_path.ok_or_else(|| {
                            ConfigError::ConfigNotSet {
                                key: String::from(DEVICE_KEY_PATH),
                            }
                        })?,
                    };
                    cmd.into_boxed()
                }
            };

        Ok(cmd)
    }
}

impl Command for CreateCertCmd {
    fn description(&self) -> String {
        format!("create a test certificate for the device {}.", self.id)
    }

    fn execute(&self, _verbose: u8) -> Result<(), anyhow::Error> {
        let config = CertConfig::default();
        let () = self.create_test_certificate(&config)?;
        let () = self.update_tedge_config()?;
        Ok(())
    }
}
impl Command for ShowCertCmd {
    fn description(&self) -> String {
        "show the device certificate".into()
    }

    fn execute(&self, _verbose: u8) -> Result<(), anyhow::Error> {
        let () = self.show_certificate()?;
        Ok(())
    }
}

impl Command for RemoveCertCmd {
    fn description(&self) -> String {
        "remove the device certificate".into()
    }

    fn execute(&self, _verbose: u8) -> Result<(), anyhow::Error> {
        let () = self.remove_certificate()?;
        let () = self.update_tedge_config()?;
        Ok(())
    }
}

struct CertConfig {
    test_cert: TestCertConfig,
}

struct TestCertConfig {
    validity_period_days: u32,
    organization_name: String,
    organizational_unit_name: String,
}

impl Default for CertConfig {
    fn default() -> Self {
        CertConfig {
            test_cert: TestCertConfig::default(),
        }
    }
}

impl Default for TestCertConfig {
    fn default() -> Self {
        TestCertConfig {
            validity_period_days: 365,
            organization_name: "Thin Edge".into(),
            organizational_unit_name: "Test Device".into(),
        }
    }
}

impl CreateCertCmd {
    fn create_test_certificate(&self, config: &CertConfig) -> Result<(), CertError> {
        check_identifier(&self.id)?;

        let cert_path = Path::new(&self.cert_path);
        let key_path = Path::new(&self.key_path);

        paths::validate_parent_dir_exists(cert_path)
            .map_err(|err| CertError::CertPathError(err))?;
        paths::validate_parent_dir_exists(key_path).map_err(|err| CertError::KeyPathError(err))?;

        // Creating files with permission 644
        let mut cert_file =
            create_new_file(&self.cert_path).map_err(|err| err.cert_context(&self.cert_path))?;
        let mut key_file =
            create_new_file(&self.key_path).map_err(|err| err.key_context(&self.key_path))?;

        let cert = new_selfsigned_certificate(&config, &self.id)?;

        let cert_pem = cert.serialize_pem()?;
        cert_file.write_all(cert_pem.as_bytes())?;
        cert_file.sync_all()?;

        // Prevent the certificate to be overwritten
        paths::set_permission(&cert_file, 0o444)?;

        {
            // Make sure the key is secret, before write
            paths::set_permission(&key_file, 0o600)?;

            // Zero the private key on drop
            let cert_key = zeroize::Zeroizing::new(cert.serialize_private_key_pem());
            key_file.write_all(cert_key.as_bytes())?;
            key_file.sync_all()?;

            // Prevent the key to be overwritten
            paths::set_permission(&key_file, 0o400)?;
        }

        Ok(())
    }

    fn update_tedge_config(&self) -> Result<(), CertError> {
        let mut config = TEdgeConfig::from_default_config()?;
        config.device.id = Some(self.id.clone());
        config.device.cert_path = Some(self.cert_path.clone());
        config.device.key_path = Some(self.key_path.clone());

        let _ = config.write_to_default_config()?;

        Ok(())
    }
}

impl ShowCertCmd {
    fn show_certificate(&self) -> Result<(), CertError> {
        let cert_path = &self.cert_path;
        let pem = read_pem(cert_path).map_err(|err| err.cert_context(cert_path))?;
        let x509 = extract_certificate(&pem)?;
        let tbs_certificate = x509.tbs_certificate;

        println!("Device certificate: {}", cert_path);
        println!("Subject: {}", tbs_certificate.subject.to_string());
        println!("Issuer: {}", tbs_certificate.issuer.to_string());
        println!(
            "Valid from: {}",
            tbs_certificate.validity.not_before.to_rfc2822()
        );
        println!(
            "Valid up to: {}",
            tbs_certificate.validity.not_after.to_rfc2822()
        );

        Ok(())
    }
}

impl RemoveCertCmd {
    fn remove_certificate(&self) -> Result<(), CertError> {
        std::fs::remove_file(&self.cert_path).or_else(ok_if_not_found)?;
        std::fs::remove_file(&self.key_path).or_else(ok_if_not_found)?;

        Ok(())
    }

    fn update_tedge_config(&self) -> Result<(), CertError> {
        let mut config = TEdgeConfig::from_default_config()?;
        config.device.id = None;

        let _ = config.write_to_default_config()?;

        Ok(())
    }
}

fn ok_if_not_found(err: std::io::Error) -> std::io::Result<()> {
    match err.kind() {
        std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(err),
    }
}

const MAX_CN_SIZE: usize = 64;

fn check_identifier(id: &str) -> Result<(), CertError> {
    if id.is_empty() {
        return Err(CertError::EmptyName);
    } else if id.len() > MAX_CN_SIZE {
        return Err(CertError::TooLongName { name: id.into() });
    } else if id.contains(char::is_control) {
        return Err(CertError::InvalidCharacter { name: id.into() });
    }

    Ok(())
}

fn extract_certificate(
    pem: &x509_parser::pem::Pem,
) -> Result<x509_parser::certificate::X509Certificate, CertError> {
    let x509 = pem.parse_x509().map_err(|err| {
        // The x509 error is wrapped into a `nom::Err`
        // and cannot be extracted without pattern matching on that type
        // So one simply extract the error as a string,
        // to avoid a dependency on the `nom` crate.
        let x509_error_string = format!("{}", err);
        CertError::X509Error(x509_error_string)
    })?;
    Ok(x509)
}

fn read_pem(path: &str) -> Result<x509_parser::pem::Pem, CertError> {
    let file = std::fs::File::open(path)?;
    let (pem, _) = x509_parser::pem::Pem::read(std::io::BufReader::new(file))?;
    Ok(pem)
}

fn create_new_file(path: &str) -> Result<File, CertError> {
    Ok(OpenOptions::new().write(true).create_new(true).open(path)?)
}

fn new_selfsigned_certificate(config: &CertConfig, id: &str) -> Result<Certificate, RcgenError> {
    let mut distinguished_name = rcgen::DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::CommonName, id);
    distinguished_name.push(
        rcgen::DnType::OrganizationName,
        &config.test_cert.organization_name,
    );
    distinguished_name.push(
        rcgen::DnType::OrganizationalUnitName,
        &config.test_cert.organizational_unit_name,
    );

    let today = Utc::now();
    let not_before = today - Duration::days(1); // Ensure the certificate is valid today
    let not_after = today + Duration::days(config.test_cert.validity_period_days.into());

    let mut params = CertificateParams::default();
    params.distinguished_name = distinguished_name;
    params.not_before = not_before;
    params.not_after = not_after;
    params.alg = &rcgen::PKCS_ECDSA_P256_SHA256; // ECDSA signing using the P-256 curves and SHA-256 hashing as per RFC 5758
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained); // IsCa::SelfSignedOnly is rejected by C8Y

    Certificate::from_params(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::fs::File;
    use tempfile::*;

    #[test]
    fn basic_usage() {
        let dir = tempdir().unwrap();
        let cert_path = temp_file_path(&dir, "my-device-cert.pem");
        let key_path = temp_file_path(&dir, "my-device-key.pem");
        let id = "my-device-id";

        let cmd = CreateCertCmd {
            id: String::from(id),
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
        };
        let verbose = 0;

        assert!(cmd.execute(verbose).err().is_none());
        assert_eq!(parse_pem_file(&cert_path).unwrap().tag, "CERTIFICATE");
        assert_eq!(parse_pem_file(&key_path).unwrap().tag, "PRIVATE KEY");
    }

    #[test]
    fn check_certificate_is_not_overwritten() {
        let cert_content = "some cert content";
        let key_content = "some key content";
        let cert_file = temp_file_with_content(cert_content);
        let key_file = temp_file_with_content(key_content);
        let id = "my-device-id";

        let cmd = CreateCertCmd {
            id: String::from(id),
            cert_path: String::from(cert_file.path().to_str().unwrap()),
            key_path: String::from(key_file.path().to_str().unwrap()),
        };
        let verbose = 0;

        assert!(cmd.execute(verbose).ok().is_none());

        let mut cert_file = cert_file.reopen().unwrap();
        assert_eq!(file_content(&mut cert_file), cert_content);

        let mut key_file = key_file.reopen().unwrap();
        assert_eq!(file_content(&mut key_file), key_content);
    }

    #[test]
    fn create_certificate_in_non_existent_directory() {
        let dir = tempdir().unwrap();
        let key_path = temp_file_path(&dir, "my-device-key.pem");

        let cmd = CreateCertCmd {
            id: "my-device-id".to_string(),
            cert_path: "/non/existent/cert/path".to_string(),
            key_path,
        };
        let verbose = 0;

        let error = cmd.execute(verbose).unwrap_err();
        let cert_error = error.downcast_ref::<CertError>().unwrap();
        assert_matches!(cert_error, CertError::CertPathError { .. });
    }

    #[test]
    fn create_key_in_non_existent_directory() {
        let dir = tempdir().unwrap();
        let cert_path = temp_file_path(&dir, "my-device-cert.pem");

        let cmd = CreateCertCmd {
            id: "my-device-id".to_string(),
            cert_path,
            key_path: "/non/existent/key/path".to_string(),
        };
        let verbose = 0;

        let error = cmd.execute(verbose).unwrap_err();
        let cert_error = error.downcast_ref::<CertError>().unwrap();
        assert_matches!(cert_error, CertError::KeyPathError { .. });
    }

    fn temp_file_path(dir: &TempDir, filename: &str) -> String {
        String::from(dir.path().join(filename).to_str().unwrap())
    }

    fn temp_file_with_content(content: &str) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        file.as_file().write_all(content.as_bytes()).unwrap();
        file
    }

    fn file_content(file: &mut File) -> String {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        content
    }

    fn parse_pem_file(path: &str) -> Result<pem::Pem, String> {
        let mut file = File::open(path).map_err(|err| err.to_string())?;
        let content = file_content(&mut file);

        pem::parse(content).map_err(|err| err.to_string())
    }
}