mod commands;
mod example_impl;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use commands::commands::get_command;
use frankenstein::TelegramApi;
use frankenstein::client_ureq::Bot;
use frankenstein::methods::AnswerCallbackQueryParams;
use frankenstein::types::{CallbackQuery, MaybeInaccessibleMessage, Message};
use lambda_runtime::{Error, LambdaEvent, service_fn, tracing};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    body: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
}

#[derive(Deserialize, Debug)]
struct Body {
    update_id: i32,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();
    example_impl::commands_impl::init();

    let func = service_fn(handle_request);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn handle_request(event: LambdaEvent<Request>) -> Result<Response, Error> {
    //TODO external crate
    let raw_body = event.payload.body;
    let body: Body = serde_json::from_str::<Body>(&raw_body).unwrap();

    let token = std::env::var("BOT_TOKEN").expect("Should have BOT_TOKEN as environment variable");
    let bot = Arc::new(Bot::new(&token));

    return match extract_request(&body, &bot) {
        Ok((chat_id, command, args)) => {
            get_command(&command)(bot, chat_id, args.to_string()).await;

            let resp = Response {
                req_id: event.context.request_id,
            };

            Ok(resp)
        }
        Err(e) => {
            println!("Error extracting: {}", e);

            let resp = Response {
                req_id: event.context.request_id,
            };

            Ok(resp)
        }
    };
}

fn extract_request<'bd>(body: &'bd Body, bot: &Bot) -> Result<(i64, String, String), Error> {
    match (body.callback_query.as_ref(), body.message.as_ref()) {
        (Some(callback_query), _) => {
            let params = AnswerCallbackQueryParams::builder()
                .callback_query_id(callback_query.id.clone())
                .text("✅ Operation processing")
                .show_alert(false)
                .build();

            let _ = bot.answer_callback_query(&params);

            let chat_id = match &callback_query.message {
                Some(MaybeInaccessibleMessage::Message(message)) => message.chat.id,
                _ => return Err(Error::from("Callback query message is inaccessible")),
            };
            let input = callback_query
                .data
                .as_ref()
                .expect("No callback request specified");

            if !input.starts_with("/") {
                return Err(Error::from("Not a bot command"));
            }

            match input.split_once(' ') {
                Some(cmd_args) => Ok((chat_id, cmd_args.0.to_string(), cmd_args.1.to_string())),
                None => Ok((chat_id, input.to_string(), input.to_string())),
            }
        }
        (_, Some(message)) => match &message.reply_to_message {
            Some(reply_to) => {
                let input = reply_to
                    .text
                    .as_ref()
                    .expect("No original message for reply");

                match input.starts_with('[') {
                    true => {
                        let cmd = input.split(&['[', ']']).collect::<Vec<&str>>()[1];
                        if !cmd.starts_with("/") {
                            return Err(Error::from("Not a bot command"));
                        }

                        let cmd_args = cmd
                            .split_once(' ')
                            .expect(format!("Incorrect command: {}", cmd).as_str());
                        let command = cmd_args.0;
                        let msg_text = message.text.as_deref().unwrap_or("");
                        let args = cmd_args.1.to_owned() + msg_text;
                        Ok((message.chat.id, command.to_string(), args))
                    }
                    false => {
                        if !input.starts_with("/") {
                            return Err(Error::from("Not a bot command"));
                        }
                        Ok((message.chat.id, input.to_string(), input.to_string()))
                    }
                }
            }
            None => {
                let input = message.text.as_ref().expect("No command input");
                if !input.starts_with("/") {
                    return Err(Error::from("Not a bot command"));
                }

                let text: Vec<&str> = message
                    .text
                    .as_ref()
                    .expect("No command input")
                    .split(' ')
                    .collect();
                let command = text[0];
                let args = *text.get(1).unwrap_or(&"");
                Ok((message.chat.id, command.to_string(), args.to_string()))
            }
        },
        (None, None) => Err(Error::from("No message or callback query found")),
    }
}
