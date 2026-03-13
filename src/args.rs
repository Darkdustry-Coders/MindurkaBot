use std::{path::PathBuf, sync::OnceLock};

use clap::Parser;

#[derive(Debug, Parser)]
pub struct AppArgs {
    #[arg(short = 'c', long, value_hint = clap::ValueHint::FilePath)]
    pub config: Option<PathBuf>,
}

static APPARGS_SINGLENTON: OnceLock<AppArgs> = OnceLock::new();

pub fn get_app_args() -> &'static AppArgs {
    APPARGS_SINGLENTON.get_or_init(|| AppArgs::parse())
}
