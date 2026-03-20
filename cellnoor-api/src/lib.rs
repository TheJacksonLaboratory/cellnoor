#![allow(uncommon_codepoints)]
#![allow(clippy::struct_field_names)]

pub mod api;
pub mod config;
mod db;
mod initial_data;
mod state;
#[cfg(any(feature = "dummy-data", test))]
mod test_state;
#[cfg(test)]
mod test_util;
