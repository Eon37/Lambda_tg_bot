use super::*;
use lambda_runtime::Context;

#[tokio::test]
async fn response_is_good_for_simple_input() {
    let id = "ID";

    let mut context = Context::default();
    context.request_id = id.to_string();

    let payload = Request {
        //TODO remove personal data
        body: r#"{
                "update_id": 764385321,
                "message": {
                    "message_id": 2,
                    "from": {
                        "id": 364257553,
                        "is_bot": false,
                        "username": "Eon37",
                        "first_name": "Eon"
                    },
                    "chat": {
                        "id": 364257553,
                        "type": "private"
                    },
                    "date": 1697032000,
                    "text": "/start"
                }
            }"#
        .to_string(),
    };
    let event = LambdaEvent { payload, context };

    let result = handle_request(event).await.unwrap();

    assert_eq!(result.req_id, id.to_string());
}

#[tokio::test]
async fn response_is_good_for_callback() {
    let id = "ID";

    let mut context = Context::default();
    context.request_id = id.to_string();

    let payload = Request {
        body: r#"{
                "update_id": 764385321,
                "callback_query": {
                    "id": "123456",
                    "from": {
                        "id": 364257553,
                        "is_bot": false,
                        "username": "Eon37",
                        "first_name": "Eon"
                    },
                    "message": {
                        "message_id": 2,
                        "from": {
                            "id": 364257553,
                            "is_bot": false,
                            "username": "Eon37",
                            "first_name": "Eon"
                        },
                        "chat": {
                            "id": 364257553,
                            "type": "private"
                        },
                        "date": 1697032000,
                        "text": "Callback message text"
                    },
                    "chat_instance": "1234567890",
                    "data": "/callback_data"
                }
            }"#
        .to_string(),
    };
    let event = LambdaEvent { payload, context };

    let result = handle_request(event).await.unwrap();

    assert_eq!(result.req_id, id.to_string());
}
