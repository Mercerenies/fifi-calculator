
mod base;
pub mod dispatch;
pub mod functional;
pub mod general;
pub mod options;
pub mod shuffle;

pub use base::{Command, CommandContext};
use functional::BinaryFunctionCommand;
use dispatch::CommandDispatchTable;

use std::collections::HashMap;

pub fn default_dispatch_table() -> CommandDispatchTable {
  let mut map: HashMap<String, Box<dyn Command + Send + Sync>> = HashMap::new();
  map.insert("+".to_string(), Box::new(BinaryFunctionCommand::new("+")));
  map.insert("-".to_string(), Box::new(BinaryFunctionCommand::new("-")));
  map.insert("*".to_string(), Box::new(BinaryFunctionCommand::new("*")));
  map.insert("/".to_string(), Box::new(BinaryFunctionCommand::new("/")));
  map.insert("pop".to_string(), Box::new(shuffle::PopCommand));
  map.insert("swap".to_string(), Box::new(shuffle::SwapCommand));
  map.insert("dup".to_string(), Box::new(shuffle::DupCommand));
  CommandDispatchTable::from_hash_map(map)
}
