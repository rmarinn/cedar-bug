uniffi::setup_scaffolding!();

use main;
use std::u32;

#[uniffi::export]
pub fn get_remaining_stack() -> String {
    main::get_remaining_stack()
}
