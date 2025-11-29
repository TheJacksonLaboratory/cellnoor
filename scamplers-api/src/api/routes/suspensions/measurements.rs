use axum_extra::routing::TypedPath;

mod read;

#[derive(TypedPath)]
#[typed_path("/measurements")]
struct MeasurementEndpoint;
