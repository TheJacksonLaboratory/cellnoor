#![expect(uncommon_codepoints)]
#![cfg_attr(test, expect(dead_code_pub_in_binary))]

pub mod api;
pub mod auth;
pub mod db;
pub mod error;
pub mod extract;
pub mod handlers;
pub mod settings;
pub mod state;
