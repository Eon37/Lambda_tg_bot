use std::collections::HashMap;
use std::fmt::Result;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, Mutex};

use frankenstein::client_ureq::Bot;

pub type CommandFn = Arc<dyn Fn(Arc<Bot>, i64, String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

static COMMANDS: LazyLock<Mutex<HashMap<&'static str, CommandFn>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    // {
    // let mut map: HashMap<&'static str, CommandFn> = HashMap::new();
    // map.insert("/start", Arc::new(|bot, chat_id, msg| Box::pin(start(bot, chat_id, msg))));
    // map.insert("/get_all", Arc::new(|bot, chat_id, msg| Box::pin(get_all(bot, chat_id, msg))));
    // map.insert("/create_callback", Arc::new(|bot, chat_id, msg| Box::pin(create_callback(bot, chat_id, msg))));
    // map.insert("/create", Arc::new(|bot, chat_id, msg| Box::pin(create(bot, chat_id, msg))));
    // map.insert("/update_callback", Arc::new(|bot, chat_id, msg| Box::pin(update_callback(bot, chat_id, msg))));
    // map.insert("/update", Arc::new(|bot, chat_id, msg| Box::pin(update(bot, chat_id, msg))));
    // map.insert("/delete", Arc::new(|bot, chat_id, msg| Box::pin(delete(bot, chat_id, msg))));
    // map.insert("/unknown", Arc::new(|bot, chat_id, msg| Box::pin(unknown(bot, chat_id, msg))));
    // map
// }); //TODO use as lib

pub fn put_command(command: &'static str, func: CommandFn) ->Result {
    COMMANDS.lock().unwrap().insert(command, func);

    Ok(())
}

pub fn get_command(command: &str) -> CommandFn {
    COMMANDS.lock().unwrap().get(command).cloned().unwrap_or_else(|| COMMANDS.lock().unwrap()["/unknown"].clone())
}