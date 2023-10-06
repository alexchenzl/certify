use thiserror::Error;
use crate::CertType;

/// General error definition for the project
#[derive(Error, Debug)]
pub enum CertifyError {
    // detailed errors
    #[error("Rcgen error: {0}")]
    RcgenError(#[from] rcgen::RcgenError),
    #[error("PEM error: {0}")]
    PemError(#[from] pem::PemError),
    #[error("Invalid cert type")]
    InvalidCertType,
}
