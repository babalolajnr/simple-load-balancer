use rustls_pemfile::{certs, rsa_private_keys, pkcs8_private_keys};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

pub fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    certs(&mut reader).collect()
}

pub fn load_keys(path: &Path) -> io::Result<Vec<PrivateKeyDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Try PKCS#8 keys first
    let keys: Vec<PrivateKeyDer<'static>> = pkcs8_private_keys(&mut reader)
        .map(|res| res.map(PrivateKeyDer::Pkcs8))
        .collect::<io::Result<Vec<_>>>()?;

    if !keys.is_empty() {
        return Ok(keys);
    }

    // Fallback to RSA keys (PKCS#1)
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let keys: Vec<PrivateKeyDer<'static>> = rsa_private_keys(&mut reader)
        .map(|res| res.map(PrivateKeyDer::Pkcs1))
        .collect::<io::Result<Vec<_>>>()?;

    Ok(keys)
}
