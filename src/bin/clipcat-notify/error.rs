use snafu::Snafu;

use clipcat::grpc::GrpcClientError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    Grpc {
        source: GrpcClientError,
    },

    #[snafu(display("Could not create tokio runtime, error: {}", source))]
    CreateTokioRuntime {
        source: std::io::Error,
    },

}


impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Error { Error::Grpc { source: err } }
}
