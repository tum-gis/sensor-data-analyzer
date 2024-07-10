use polars::error::PolarsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ErosbagError(#[from] erosbag::Error),
    #[error(transparent)]
    EpointError(#[from] epoint::Error),
    #[error(transparent)]
    EpointIoError(#[from] epoint::io::Error),
    #[error(transparent)]
    EmeshError(#[from] emesh::Error),
    #[error(transparent)]
    EmeshConverterError(#[from] emesh_converter::Error),
    #[error(transparent)]
    EgraphicsIoError(#[from] egraphics::io::Error),

    #[error(transparent)]
    StdIoResult(#[from] std::io::Error),
    #[error(transparent)]
    DieselResult(#[from] diesel::result::Error),
    #[error(transparent)]
    PolarsResult(#[from] PolarsError),
}
