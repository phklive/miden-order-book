use crate::order::Order;

#[derive(Debug, PartialEq, Eq)]
pub enum OrderError {
    AssetsNotMatching,
    TooFewSourceAssets,
    TooManyTargetAssets,
    FailedFill(Order),
    MissingId,
    InternalError(String),
}
