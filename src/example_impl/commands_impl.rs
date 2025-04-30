use frankenstein::TelegramApi;
use frankenstein::client_ureq::Bot;
use frankenstein::methods::SendMessageParams;
use frankenstein::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::aws::dynamodb;
use crate::commands::commands;
//TODO feature?
static START_TEXT: &str =
    "Hello! with this bot you can show the list of items, edit or delete them";
static CREATE_CALLBACK_TEXT: &str =
    "Please, reply to this message with the text you want to create";
static UPDATE_CALLBACL_TEXT: &str =
    "Please, reply to this message with the text you want to update";
static CREATE_INCORRECT_TEXT: &str = "Incorrect format. Please, send (brackets not needed, but semicolon's required): \"/create <text>\"";
static UPDATE_INCORRECT_TEXT: &str = "Incorrect format. Please, send (brackets not needed, but semicolon's required): \"/update <partition_key>;<sort_key>;<text>\"";
static DELETE_INCORRECT_TEXT: &str = "Incorrect format. Please, send (brackets not needed, but semicolon's required): \"/delete <partition_key>;<sort_key>\"";

pub fn init() {
    commands::put_command(
        "/start",
        Arc::new(|bot, chat_id, msg| Box::pin(start(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/get_all",
        Arc::new(|bot, chat_id, msg| Box::pin(get_all(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/create_callback",
        Arc::new(|bot, chat_id, msg| Box::pin(create_callback(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/create",
        Arc::new(|bot, chat_id, msg| Box::pin(create(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/update_callback",
        Arc::new(|bot, chat_id, msg| Box::pin(update_callback(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/update",
        Arc::new(|bot, chat_id, msg| Box::pin(update(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/delete",
        Arc::new(|bot, chat_id, msg| Box::pin(delete(bot, chat_id, msg))),
    )
    .unwrap();
    commands::put_command(
        "/unknown",
        Arc::new(|bot, chat_id, msg| Box::pin(unknown(bot, chat_id, msg))),
    )
    .unwrap();
}

async fn unknown(bot: Arc<Bot>, chat_id: i64, args: String) {
    send_message(&bot, chat_id, "Unknown command", vec![], true);
}

async fn start(bot: Arc<Bot>, chat_id: i64, args: String) {
    send_message(
        &bot,
        chat_id,
        START_TEXT,
        vec![vec![
            InlineKeyboardButton::builder()
                .text("Show list")
                .callback_data("/get_all")
                .build(),
            InlineKeyboardButton::builder()
                .text("Create item")
                .callback_data("/create_callback")
                .build(),
        ]],
        false,
    );
}

async fn get_all(bot: Arc<Bot>, chat_id: i64, args: String) {
    let client = dynamodb::connect().await.unwrap();

    let mut iterator = None;

    loop {
        let (items, new_iterator) = dynamodb::find_all(&client, iterator).await.unwrap();
        iterator = new_iterator;

        for item in items {
            println!("{:?}", item);
            let pk = item.get(&dynamodb::PK.clone()).unwrap().as_s().unwrap(); //TODO dynamic?
            let sk = item.get(&dynamodb::SK.clone()).unwrap().as_s().unwrap();
            let text = item.get(&dynamodb::TEXT.clone()).unwrap().as_s().unwrap();

            send_message(
                &bot,
                chat_id,
                &format!("pk: {}\nsk: {}\ntext: {}", pk, sk, text),
                vec![vec![
                    InlineKeyboardButton::builder()
                        .text("Edit item")
                        .callback_data(format!("/update_callback {}~{}~", pk, sk))
                        .build(),
                    InlineKeyboardButton::builder()
                        .text("Delete item")
                        .callback_data(format!("/delete {}~{}", pk, sk))
                        .build(),
                ]],
                true,
            );
        }

        if iterator.is_none() {
            break;
        }
    }
}

async fn create_callback(bot: Arc<Bot>, chat_id: i64, args: String) {
    send_message(
        &bot,
        chat_id,
        &("[/create ]\n\n".to_owned() + CREATE_CALLBACK_TEXT),
        vec![],
        false,
    );
}

async fn create(bot: Arc<Bot>, chat_id: i64, args: String) {
    if args == "" {
        send_message(&bot, chat_id, CREATE_INCORRECT_TEXT, vec![], true);
        return;
    }

    let client = dynamodb::connect().await.unwrap();
    let mut hasher = DefaultHasher::new(); //TODO check length
    args.hash(&mut hasher);

    let pk = hasher.finish().to_string();
    let sk = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string();

    match dynamodb::put(&client, &pk, &sk, &args).await {
        Ok(_) => send_message(&bot, chat_id, "Item created", vec![], true),
        Err(e) => {
            println!("Error: {}", e);
            send_message(&bot, chat_id, "Error creating item!", vec![], true)
        }
    }
}

async fn update_callback(bot: Arc<Bot>, chat_id: i64, args: String) {
    send_message(
        &bot,
        chat_id,
        &(format!("[/update {}]\n\n", args) + UPDATE_CALLBACL_TEXT),
        vec![],
        false,
    );
}

async fn update(bot: Arc<Bot>, chat_id: i64, args: String) {
    if args == "" {
        send_message(&bot, chat_id, UPDATE_INCORRECT_TEXT, vec![], true);
        return;
    }

    let client = dynamodb::connect().await.unwrap();

    let upd: Vec<&str> = args.split('~').collect();
    println!("{:?}", upd);

    match dynamodb::put(&client, &upd[0], &upd[1], &upd[2]).await {
        Ok(_) => send_message(&bot, chat_id, "Item updated", vec![], true),
        Err(e) => {
            println!("Error: {}", e);
            send_message(&bot, chat_id, "Error updating item!", vec![], true)
        }
    }
}

async fn delete(bot: Arc<Bot>, chat_id: i64, args: String) {
    if args == "" {
        send_message(&bot, chat_id, DELETE_INCORRECT_TEXT, vec![], true);
        return;
    }

    let client = dynamodb::connect().await.unwrap();

    let dlt: Vec<&str> = args.split('~').collect();

    match dynamodb::delete(&client, dlt[0], dlt[1]).await {
        Ok(_) => send_message(&bot, chat_id, "Item deleted", vec![], true),
        Err(e) => {
            println!("Error: {}", e);
            send_message(&bot, chat_id, "Error deleting item!", vec![], true)
        }
    }
}

fn send_message(
    bot: &Bot,
    chat_id: i64,
    text: &str,
    mut buttons: Vec<Vec<InlineKeyboardButton>>,
    include_back_btn: bool,
) {
    if include_back_btn {
        let start_btn = vec![
            InlineKeyboardButton::builder()
                .text("<< Back to menu")
                .callback_data("/start")
                .build(),
        ];

        buttons.push(start_btn);
    }

    let params: SendMessageParams = SendMessageParams::builder()
        .chat_id(chat_id)
        .text(text)
        .reply_markup(ReplyMarkup::InlineKeyboardMarkup(
            InlineKeyboardMarkup::builder()
                .inline_keyboard(buttons)
                .build(),
        ))
        .build();

    bot.send_message(&params).unwrap();
}
