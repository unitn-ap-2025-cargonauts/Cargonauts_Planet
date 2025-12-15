//! Wrapper fot the loggin module defined in the common crate.

use common_game::components::planet::PlanetState;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};

/// Logs an event with a structured payload, automatically generating channel-specific keys.
///
/// Transforms a vector of strings into a `Payload` object by creating key-value pairs where each key
/// is dynamically generated based on the channel type and the string's position in the vector.
/// The generated `LogEvent` is then emitted for logging.
///
/// # Arguments
///
/// * `sender_id` - The ID of the sender (converted to `u64`)
/// * `receiver_type` - The type of the receiver actor
/// * `receiver_id` - The ID of the receiver (converted to `String`)
/// * `event_type` - The type of event being logged
/// * `channel` - The communication channel that determines the key prefix (Error, Info, Debug, Warning, Trace)
/// * `payload` - A vector of strings to be logged as values
///
/// # Key Generation
///
/// For each string in the payload vector, a key is generated following the pattern:
/// `"<channel_name> #<index> detail:"` where:
/// - `<channel_name>`: One of "Error", "Info", "Debug", "Warning", or "Trace" (determined by the `channel` parameter)
/// - `<index>`: The zero-based position of the string within the vector
/// - `detail:`: A literal suffix appended to each key
///
/// # Side Effects
///
/// - Constructs a `Payload` object from the input strings with auto-generated keys
/// - Creates a `LogEvent` with `ActorType::planet` as the sender actor type
/// - Emits the constructed `LogEvent` for logging
///
/// # Example
///
/// ```text
/// let payload = vec!["Connection failed".to_string(), "Timeout occurred".to_string()];
///
/// log_message(
///     42,
///     ActorType::Server,
///     "server_001",
///     EventType::Error,
///     Channel::Error,
///     payload
/// );
/// // Logs with keys:
/// // "Error #0 detail:" => "Connection failed"
/// // "Error #1 detail:" => "Timeout occurred"
/// ```
fn log_message(
    sender_id: impl Into<u64>,
    receiver_type: ActorType,
    receiver_id: impl Into<String>,
    event_type: EventType,
    channel: Channel,
    payload: Vec<String>,
) {
    // Create the planet object
    let mut payload_object = Payload::new();

    payload
        .iter()
        .enumerate()
        .map(|(index, inserted_string)| match channel {
            Channel::Error => (format!("Error #{} detail:", index), inserted_string),
            Channel::Info => (format!("Info #{} detail:", index), inserted_string),
            Channel::Debug => (format!("Debug #{} detail:", index), inserted_string),
            Channel::Warning => (format!("Warning #{} detail:", index), inserted_string),
            Channel::Trace => (format!("Trace #{} detail:", index), inserted_string),
        })
        .for_each(|(key_val, string_req)| {
            payload_object.insert(key_val, string_req.to_string());
        });

    LogEvent::new(
        ActorType::Planet,
        sender_id,
        receiver_type,
        receiver_id,
        event_type,
        channel,
        payload_object,
    )
        .emit();
}

/// Logs an event with a structured key-value payload.
///
/// Creates a `LogEvent` from the provided parameters and emits it for logging.
/// Transforms a vector of key-value pairs into a `Payload` object before emission.
///
/// # Arguments
///
/// * `sender_id` - The ID of the sender (converted to `u64`)
/// * `receiver_type` - The type of the receiver actor
/// * `receiver_id` - The ID of the receiver (converted to `String`)
/// * `event_type` - The type of event being logged
/// * `channel` - The communication channel through which the event is transmitted
/// * `payload` - A vector of tuples containing key-value pairs where both key and value are strings
///
/// # Side Effects
///
/// - Constructs a `Payload` object from the input key-value pairs
/// - Creates a `LogEvent` with `ActorType::planet` as the sender actor type
/// - Emits the constructed `LogEvent` for logging
///
/// # Example
///
/// ```text
/// let payload = vec![
///     ("event_name".to_string(), "user_login".to_string()),
///     ("timestamp".to_string(), "2024-01-15".to_string()),
/// ];
///
/// log_message_with_key(
///     123,
///     ActorType::User,
///     "user_456",
///     EventType::Login,
///     Channel::Auth,
///     payload
/// );
/// ```
fn log_message_with_key(
    sender_id: impl Into<u64>,
    receiver_type: ActorType,
    receiver_id: impl Into<String>,
    event_type: EventType,
    channel: Channel,
    payload: Vec<(String, String)>,
) {
    // Create the planet object
    let mut payload_object = Payload::new();

    payload.iter().for_each(|(payload_key, payload_content)| {
        payload_object.insert(payload_key.to_string(), payload_content.to_string());
    });

    LogEvent::new(
        ActorType::Planet,
        sender_id,
        receiver_type,
        receiver_id,
        event_type,
        channel,
        payload_object,
    )
        .emit();
}

/// Macro that generates specialized logging functions for each communication channel.
///
/// This macro creates two variants of logging functions for each channel type provided:
/// 1. `log_for_channel_<name>()` - Logs with auto-generated channel-specific keys
/// 2. `log_for_channel_with_key_<name>()` - Logs with custom key-value pairs
///
/// # Arguments
///
/// * `$Enum` - The enum variant of the `Channel` type (e.g., `Error`, `Info`, `Debug`, `Warning`, `Trace`)
/// * `$name` - A string literal representing the channel name (used in the generated function name in lowercase)
///
/// This generates:
/// ```text
/// - log_for_channel_error(sender_id, receiver_type, receiver_id, event_type, payload)
/// - log_for_channel_with_key_error(sender_id, receiver_type, receiver_id, event_type, payload)
/// - log_for_channel_info(sender_id, receiver_type, receiver_id, event_type, payload)
/// - log_for_channel_with_key_info(sender_id, receiver_type, receiver_id, event_type, payload)
/// - log_for_channel_debug(sender_id, receiver_type, receiver_id, event_type, payload)
/// - log_for_channel_with_key_debug(sender_id, receiver_type, receiver_id, event_type, payload)
/// ```
macro_rules! specialize_channel_func {
    ( $( $Enum:ident => $name:literal ),* ) => {
        $(
            paste::paste!(
                #[allow(unused)]
                pub fn [<log_for_channel_ $name:lower>](
                    sender_id: impl Into<u64>,
                    receiver_type: ActorType,
                    receiver_id: impl Into<String>,
                    event_type: EventType,
                    payload: Vec<String>,
                ) {
                    log_message(
                        sender_id.into(),
                        receiver_type,
                        receiver_id.into(),
                        event_type,
                        Channel::$Enum,
                        payload
                    )
                }
                #[allow(unused)]
                pub fn [<log_for_channel_with_key_ $name:lower>] (
                    sender_id: impl Into<u64>,
                    receiver_type: ActorType,
                    receiver_id: impl Into<String>,
                    event_type: EventType,
                    payload: Vec<(String, String)>,
                ) {
                    log_message_with_key(
                        sender_id.into(),
                        receiver_type,
                        receiver_id.into(),
                        event_type,
                        Channel::$Enum,
                        payload
                    )
                }
            );
        )*
    };
}

// Define scope of the macro
specialize_channel_func!(
    Error => "Error",
    Warning => "Warning",
    Info => "Info",
    Debug => "Debug",
    Trace => "Trace"
);

pub fn drop_planet_state_as_string(planet_state: &PlanetState) -> String {
    let stringify_rocket = |option_rocket: bool| -> String {
        match option_rocket {
            true => "Some(rocket)".to_string(),
            false => "None".to_string(),
        }
    };
    format!(
        "id: {}, EnergyCell: is charged? {}, rocket: {}",
        planet_state.id(),
        planet_state.cell(0).is_charged(),
        stringify_rocket(planet_state.has_rocket())
    )
}

pub fn drop_planet_state_fields_as_vector(planet_state: &PlanetState) -> Vec<(String, String)> {
    let stringify_rocket = |option_rocket: bool| -> String {
        match option_rocket {
            true => "Some(rocket)".to_string(),
            false => "None".to_string(),
        }
    };
    vec![
        ("id".to_string(), format!("{:?}", planet_state.id())),
        (
            "energy_cell.is_charged()".to_string(),
            format!("{}", planet_state.cell(0).is_charged()),
        ),
        (
            "rocket".to_string(),
            stringify_rocket(planet_state.has_rocket()).to_string(),
        ),
    ]
}

pub fn append_info_to_state(
    state: &PlanetState,
    input_vec: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut vec = Vec::<(String, String)>::new();

    input_vec.iter().for_each(|(key, value)| {
        vec.push((key.to_string(), value.to_string()));
    });

    // Load the planet state
    let planet_state = drop_planet_state_fields_as_vector(state);

    planet_state.iter().for_each(|(key, value)| {
        vec.push((key.to_string(), value.to_string()));
    });

    vec
}
