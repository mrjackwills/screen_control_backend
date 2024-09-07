use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MessageValues {
    Valid(ParsedMessage, String),
    Invalid(ErrorData),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScreenStatus {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "name", content = "body")]
pub enum ParsedMessage {
    Status,
    ScreenOn,
    ScreenOff,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct StructuredMessage {
    data: Option<ParsedMessage>,
    error: Option<ErrorData>,
    unique: String,
}

// TODO - this is, at the moment, pointless
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "error", content = "message")]
pub enum ErrorData {
    Something(String),
}

// Change this to a Result<MessageValues, AppError>?
pub fn to_struct(input: &str) -> Option<MessageValues> {
    if let Ok(data) = serde_json::from_str::<StructuredMessage>(input) {
        if let Some(message) = data.error {
            return Some(MessageValues::Invalid(message));
        }
        if let Some(message) = data.data {
            return Some(MessageValues::Valid(message, data.unique));
        }
        None
    } else {
        let error_serialized = serde_json::from_str::<ErrorData>(input);
        error_serialized.map_or(None, |data| Some(MessageValues::Invalid(data)))
    }
}

/// message_incoming
///
/// cargo watch -q -c -w src/ -x 'test message_incoming -- --test-threads=1 --nocapture'
#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_is_none(json: &str) {
        let result = to_struct(json);
        assert!(result.is_none());
    }

    fn test_is_some(json: &str) {
        let result = to_struct(json);
        assert!(result.is_some());
        match result.unwrap() {
            MessageValues::Valid(_, _) => (),
            MessageValues::Invalid(_) => unreachable!("this indicates the test has failed"),
        }
    }

    #[test]
    fn message_incoming_parse_invalid() {
        let data = "";
        let result = to_struct(data);
        assert!(result.is_none());

        let data = "{}";
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_status() {
        // body included
        test_is_none(
            r#"{ "data": { "name": "status", "body": { "minute": 6 } }, "unique": "random_string"}"#,
        );

        // no unique
        test_is_none(r#"{ "data": { "name": "status" } }"#);

        // valid message
        test_is_some(r#"{ "data": { "name": "status" }, "unique": "random_string"}"#);
    }

    #[test]
    fn message_incoming_parse_screen() {
        // valid screen on
        test_is_some(r#"{ "data": { "name": "screen_on" }, "unique":"true"}"#);

        // valid screen off
        test_is_some(r#"{ "data": { "name": "screen_off" }, "unique":"true"}"#);
    }
}
