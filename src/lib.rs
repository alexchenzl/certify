mod cert;
mod error;

pub use cert::{CertInfo, CertSigAlgo, CertType, CA};
pub use error::CertifyError;

// re-exports
pub use rcgen::KeyPair;

/// Generate CA cert
pub fn generate_ca(
    country: &str,
    org: &str,
    cn: &str,
    sig_algo: CertSigAlgo,
    pem_str: Option<&str>,
    days: Option<i64>,
) -> Result<(String, String), CertifyError> {
    let params = CertInfo::new(
        Vec::<String>::new(),
        Vec::<String>::new(),
        country,
        org,
        cn,
        days,
        sig_algo,
    );
    let keypair = match pem_str {
        Some(v) => Some(KeyPair::from_pem(v)?),
        None => None,
    };
    let ca = params.ca_cert(keypair)?;
    Ok((ca.serialize_pem().unwrap(), ca.serialize_private_key_pem()))
}

/// generate cert signed by the CA
#[allow(clippy::too_many_arguments)]
pub fn generate_cert(
    ca: &CA,
    domains: Vec<impl Into<String>>,
    country: &str,
    org: &str,
    cn: &str,
    sig_algo: CertSigAlgo,
    pem_str: Option<&str>,
    cert_type: u32,
    days: Option<i64>,
) -> Result<(String, String), CertifyError> {
    let params = CertInfo::new(
        domains,
        vec!["127.0.0.1", "::1"],
        country,
        org,
        cn,
        days,
        sig_algo,
    );
    let keypair = match pem_str {
        Some(v) => Some(KeyPair::from_pem(v)?),
        None => None,
    };
    // let cert = if is_client {
    //     params.client_cert(keypair)?
    // } else {
    //     params.server_cert(keypair)?
    // };

    let cert = match cert_type {
        0 => params.server_cert(keypair)?,
        1 => params.client_cert(keypair)?,
        2 => params.p2p_cert(keypair)?,
        _ => return Err(CertifyError::InvalidCertType(cert_type)),
    };

    let (cert_pem, key_pem) = ca.sign_cert(&cert)?;
    Ok((cert_pem, key_pem))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ca_cert_should_work() -> Result<(), CertifyError> {
        let (cert, key) = gen_ca(None)?;
        let ca = CA::load(&cert, &key)?;
        let cert1 = ca.serialize_pem()?;

        assert_eq!(cert, cert1);
        Ok(())
    }

    #[test]
    fn generate_ca_cert_with_existing_key_should_work() -> Result<(), CertifyError> {
        let key_pem = include_str!("fixtures/ca_key.pem");
        let ca_pem = include_str!("fixtures/ca_cert.pem");
        let (cert, key) = gen_ca(Some(key_pem))?;
        let ca = CA::load(&cert, &key)?;
        let cert1 = ca.serialize_pem()?;

        assert_eq!(cert, cert1);
        assert_eq!(key_pem, key);

        let ca = CA::load(ca_pem, &key)?;
        let cert2 = ca.serialize_pem()?;

        assert_eq!(ca_pem, cert2);

        Ok(())
    }

    #[test]
    fn generate_server_cert_with_ca_should_work() -> Result<(), CertifyError> {
        let (cert, key) = gen_ca(None)?;
        let ca = CA::load(&cert, &key)?;
        let (server_cert, server_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "US",
            "Domain Domain Inc.",
            "API Server",
            CertSigAlgo::ED25519,
            None,
            0,
            Some(365),
        )?;

        println!("{}\n{}", server_cert, server_key);

        Ok(())
    }

    #[test]
    fn generate_server_cert_with_existing_ca_and_key_should_work() -> Result<(), CertifyError> {
        let key_pem = include_str!("fixtures/ca_key.pem");
        let ca_pem = include_str!("fixtures/ca_cert.pem");
        let server_key_pem = include_str!("fixtures/server_key.pem");

        let ca = CA::load(ca_pem, key_pem)?;
        let (server_cert, server_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "US",
            "Domain Domain Inc.",
            "API Server",
            CertSigAlgo::ED25519,
            Some(server_key_pem),
            0,
            Some(365),
        )?;

        assert_eq!(&server_key, server_key_pem);

        println!("{}\n{}", server_cert, server_key);

        Ok(())
    }

    #[test]
    fn generate_client_cert_with_ca_should_work() -> Result<(), CertifyError> {
        let (cert, key) = gen_ca(None)?;
        let ca = CA::load(&cert, &key)?;
        let (client_cert, client_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "CA",
            "macos",
            "awesome_device_id",
            CertSigAlgo::ED25519,
            None,
            1,
            Some(365),
        )?;

        println!("{}\n{}", client_cert, client_key);

        Ok(())
    }

    #[test]
    fn generate_client_cert_with_existing_ca_and_key_should_work() -> Result<(), CertifyError> {
        let key_pem = include_str!("fixtures/ca_key.pem");
        let ca_pem = include_str!("fixtures/ca_cert.pem");
        let client_key_pem = include_str!("fixtures/client_key.pem");

        let ca = CA::load(ca_pem, key_pem)?;

        let (client_cert, client_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "CA",
            "macos",
            "awesome_device_id",
            CertSigAlgo::ED25519,
            Some(client_key_pem),
            1,
            Some(365),
        )?;

        assert_eq!(&client_key, client_key_pem);
        println!("{}\n{}", client_cert, client_key);

        Ok(())
    }

    #[test]
    fn generate_p2p_cert_with_ca_should_work() -> Result<(), CertifyError> {
        let (cert, key) = gen_ca(None)?;
        let ca = CA::load(&cert, &key)?;
        let (p2p_cert, p2p_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "US",
            "Domain Domain Inc.",
            "API Server",
            CertSigAlgo::ED25519,
            None,
            0,
            Some(365),
        )?;

        println!("{}\n{}", p2p_cert, p2p_key);

        Ok(())
    }

    #[test]
    fn generate_p2p_cert_with_existing_ca_and_key_should_work() -> Result<(), CertifyError> {
        let key_pem = include_str!("fixtures/ca_key.pem");
        let ca_pem = include_str!("fixtures/ca_cert.pem");
        let p2p_key_pem = include_str!("fixtures/server_key.pem");

        let ca = CA::load(ca_pem, key_pem)?;
        let (p2p_cert, p2p_key) = generate_cert(
            &ca,
            vec!["app.domain.com"],
            "US",
            "Domain Domain Inc.",
            "API Server",
            CertSigAlgo::ED25519,
            Some(p2p_key_pem),
            0,
            Some(365),
        )?;

        assert_eq!(&p2p_key, p2p_key_pem);

        println!("{}\n{}", p2p_key, p2p_key);

        Ok(())
    }

    fn gen_ca(pem: Option<&str>) -> Result<(String, String), CertifyError> {
        generate_ca(
            "US",
            "Domain Domain Inc.",
            "Domain CA",
            CertSigAlgo::ED25519,
            pem,
            None,
        )
    }

    // fn write_file(name: &str, content: &str) {
    //     use std::io::Write;
    //     let mut file = std::fs::File::create(name).unwrap();
    //     file.write_all(content.as_bytes()).unwrap();
    // }
}
