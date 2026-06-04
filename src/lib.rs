pub mod agent;
pub mod audit;
pub mod cli;
pub mod config;
pub mod error;
pub mod installer;
pub mod progress;
pub mod registry;
pub mod scanner;

mod main_impl;

pub use agent::AgentKind;
pub use audit::{AuditReport, Verdict, audit as audit_run};
pub use cli::{Cli, Commands};
pub use config::Config;
pub use error::{Result, SkillHubError};
pub use installer::Installer;
pub use registry::{Registry, RegistryClient, Skill, SkillRef};

pub fn run() -> anyhow::Result<i32> {
    main_impl::run_with_args(std::env::args_os())
}

pub fn run_with_args<I, T>(args: I) -> anyhow::Result<i32>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    main_impl::run_with_args(args)
}
