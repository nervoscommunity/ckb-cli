use std::path::PathBuf;

use super::super::{from_matches, CliSubCommand};
use crate::utils::printer::Printable;
use bytes::Bytes;
use ckb_core::script::Script;
use ckb_sdk::{with_rocksdb, HttpRpcClient, ScriptManager};
use clap::{App, Arg, ArgMatches, SubCommand};
use faster_hex::hex_decode;
use jsonrpc_types::Script as RpcScript;
use numext_fixed_hash::H256;

pub struct LocalScriptSubCommand<'a> {
    rpc_client: &'a mut HttpRpcClient,
    db_path: PathBuf,
}

impl<'a> LocalScriptSubCommand<'a> {
    pub fn new(rpc_client: &'a mut HttpRpcClient, db_path: PathBuf) -> LocalScriptSubCommand<'a> {
        LocalScriptSubCommand {
            rpc_client,
            db_path,
        }
    }

    pub fn subcommand() -> App<'static, 'static> {
        SubCommand::with_name("script")
            .about("Local script management")
            .subcommands(vec![
                SubCommand::with_name("add")
                    .arg(
                        Arg::with_name("code-hash")
                            .long("code-hash")
                            .takes_value(true)
                            .required(true)
                            .help("Code hash (CellOutput.data.hash)"),
                    )
                    .arg(
                        Arg::with_name("args")
                            .long("args")
                            .takes_value(true)
                            .multiple(true)
                            .help("Script arguments"),
                    ),
                SubCommand::with_name("remove").arg(
                    Arg::with_name("script-hash")
                        .long("script-hash")
                        .takes_value(true)
                        .required(true)
                        .help("Script hash"),
                ),
                SubCommand::with_name("show").arg(
                    Arg::with_name("script-hash")
                        .long("script-hash")
                        .takes_value(true)
                        .required(true)
                        .help("Script hash"),
                ),
                SubCommand::with_name("list"),
            ])
    }
}

impl<'a> CliSubCommand for LocalScriptSubCommand<'a> {
    fn process(&mut self, matches: &ArgMatches) -> Result<Box<dyn Printable>, String> {
        match matches.subcommand() {
            ("add", Some(m)) => {
                let code_hash: H256 = from_matches(m, "code-hash");
                let args_result: Result<Vec<Bytes>, String> = m
                    .values_of_lossy("args")
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .map(|data| {
                        let data_hex = if data.starts_with("0x") || data.starts_with("0X") {
                            &data[2..]
                        } else {
                            &data
                        };
                        let mut data_bytes = vec![0; data_hex.len() / 2];
                        hex_decode(data_hex.as_bytes(), &mut data_bytes)
                            .map_err(|err| format!("parse to-data failed: {:?}", err))?;
                        Ok(Bytes::from(data_bytes))
                    })
                    .collect();
                let args = args_result?;
                let script = Script::new(args, code_hash);
                let script_hash = script.hash();
                with_rocksdb(&self.db_path, None, move |db| {
                    ScriptManager::new(db).add(script).map_err(Into::into)
                })
                .map_err(|err| format!("{:?}", err))?;
                Ok(Box::new(serde_json::to_string(&script_hash).unwrap()))
            }
            ("remove", Some(m)) => {
                let script_hash: H256 = from_matches(m, "script-hash");
                with_rocksdb(&self.db_path, None, |db| {
                    ScriptManager::new(db)
                        .remove(&script_hash)
                        .map_err(Into::into)
                })
                .map_err(|err| format!("{:?}", err))?;
                Ok(Box::new("true".to_string()))
            }
            ("show", Some(m)) => {
                let script_hash: H256 = from_matches(m, "script-hash");
                let script = with_rocksdb(&self.db_path, None, |db| {
                    ScriptManager::new(db).get(&script_hash).map_err(Into::into)
                })
                .map_err(|err| format!("{:?}", err))?;
                let rpc_script: RpcScript = script.into();
                Ok(Box::new(serde_json::to_string(&rpc_script).unwrap()))
            }
            ("list", _) => {
                let scripts = with_rocksdb(&self.db_path, None, |db| {
                    ScriptManager::new(db).list().map_err(Into::into)
                })
                .map_err(|err| format!("{:?}", err))?;
                let rpc_scripts: Vec<RpcScript> = scripts.into_iter().map(Into::into).collect();
                Ok(Box::new(serde_json::to_string(&rpc_scripts).unwrap()))
            }
            _ => Err(matches.usage().to_owned()),
        }
    }
}
