//! External service clients (AWS Cognito, AWS S3).

pub mod cognito;
pub mod s3;

pub use cognito::CognitoService;
pub use s3::S3Service;
