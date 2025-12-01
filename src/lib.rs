use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "auto-install-tools")] {
        pub mod install;
        pub mod pkg;
    }
}

pub mod cli;
pub mod registry;

pub mod env;
pub mod process;
pub mod util;
