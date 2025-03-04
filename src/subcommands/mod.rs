pub mod rpc;
#[cfg(unix)]
pub mod tui;
pub mod wallet;

#[cfg(feature = "local")]
pub mod local;

#[cfg(unix)]
pub use self::tui::TuiSubCommand;

#[cfg(feature = "local")]
pub use local::{
    LocalCellInputSubCommand, LocalCellSubCommand, LocalKeySubCommand, LocalScriptSubCommand,
    LocalSubCommand, LocalTxSubCommand,
};

pub use rpc::RpcSubCommand;
pub use wallet::{
    start_index_thread, IndexController, IndexRequest, IndexResponse, IndexThreadState,
    WalletSubCommand,
};

use clap::ArgMatches;
use serde::de::DeserializeOwned;

use crate::utils::printer::Printable;

pub trait CliSubCommand {
    fn process(&mut self, matches: &ArgMatches) -> Result<Box<dyn Printable>, String>;
}

fn from_string<T: DeserializeOwned>(source: String) -> T {
    let value = serde_json::Value::String(source);
    serde_json::from_value(value).unwrap()
}

fn from_matches<T>(matches: &ArgMatches, name: &str) -> T
where
    T: DeserializeOwned,
{
    from_string(matches.value_of(name).unwrap().to_string())
}

fn from_matches_opt<T>(matches: &ArgMatches, name: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    matches
        .value_of(name)
        .map(|value_str| from_string(value_str.to_string()))
        .unwrap_or(None)
}
