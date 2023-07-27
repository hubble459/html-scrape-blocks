#[cfg(all(feature = "chrono", feature = "time"))]
compile_error!("Feature chrono and time are mutually exclusive and cannot be enabled together");

#[macro_use]
extern crate log;

pub mod model;
pub mod util;