// Documentation URL: https://spacetimedb.com/docs/client-languages/rust/rust-sdk-quickstart-guide

mod module_bindings;
use module_bindings::*;

use spacetimedb_sdk::{
    identity::{
        load_credentials,
        once_on_connect,
        save_credentials,
        Credentials,
        Identity
    },
    on_subscription_applied,
    reducer::Status,
    subscribe,
    table::{
        TableType,
        TableWithPrimaryKey
    },
};

// Global Variable
const CREDS_DIR: &str = ".spacetime_chat";
const SPACETIMEDB_URI: &str = "http://localhost:3000";
const DB_NAME: &str = "quick-chat";

fn main() {
    register_callbacks();
    connect_to_db();
    subscribe_to_tables();
    user_input_loop();
}

//Register all the callbacks our app will use to respond to database events
fn register_callbacks() {
    once_on_connect(on_connected); // Save received credentials to file
    User::on_insert(on_user_inserted); // Notify when new user joins
    User::on_update(on_user_updated); // Notify when a user's status changes
    Message::on_insert(on_message_inserted); // Notify when a new message is received
    on_subscription_applied(on_sub_applied); // Print backlog in timestamp order
    on_set_name(on_name_set); // Print a warning when we fail to set our name
    on_send_message(on_message_sent); // Print a warning when we fail to send a message
}

// Save our credentials
fn on_connected(creds: &Credentials) {
    if let Err(e) = save_credentials(CREDS_DIR, creds) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
}

// If a user is online, print a notification
fn on_user_inserted(user: &User, _: Option<&ReducerEvent>) {
    if user.online {
        println!("User {} connected", user_name_or_identity(user));
    }
}

fn user_name_or_identity(user: &User) -> String {
    user.name
        .clone()
        .unwrap_or_else(|| identity_leading_hex(&user.identity))
}

fn identity_leading_hex(id: &Identity) -> String {
    hex::encode(&id.bytes()[0..8])
}

// Print a notification about name & status changes
fn on_user_updated(old: &User, new: &User, _: Option<&ReducerEvent>) {
    if old.name != new.name {
        println!(
            "User {} renamed to {}.",
            user_name_or_identity(old),
            user_name_or_identity(new)
        );
    } 
    if old.online && !new.online {
        println!("User {} disconnected.", user_name_or_identity(new));
    }
    if !old.online && new.online {
        println!("User {} connected.", user_name_or_identity(new));
    }
}

// Print new messages
fn on_message_inserted(message: &Message, reducer_event: Option<&ReducerEvent>) {
    if reducer_event.is_some() {
        print_message(message);
    }
}

fn print_message(message: &Message) {
    let sender = User::filter_by_identity(message.sender.clone())
        .map(|u| user_name_or_identity(&u))
        .unwrap_or_else(|| "unknown".to_string());
    println!("{}: {}", sender, message.text);
}

// Print all past messages
fn on_sub_applied() {
    let mut messages = Message::iter().collect::<Vec<_>>();
    messages.sort_by_key(|m| m.sent);
    for message in messages {
        print_message(&message)
    }
}

// Print warning if user is unable to change username
fn on_name_set(_sender: &Identity, status: &Status, name: &String) {
    if let Status::Failed(err) = status {
        eprintln!("Failed to change name to {:?}: {}", name, err);
    }
}

// Print warning if message fails to send
fn on_message_sent(_sender: &Identity, status: &Status, text: &String) {
    if let Status::Failed(err) = status {
        eprintln!("Failed to send message {:?}: {}", text, err);
    }
}

// Load credentials from a file and connect to the database
fn connect_to_db() {
    connect(
        SPACETIMEDB_URI,
        DB_NAME,
        load_credentials(CREDS_DIR).expect("Error reading stored credentials"),
    )
    .expect("Failed to connect");
}

// Register subscriptions for all rows of both tables
fn subscribe_to_tables() {
    subscribe(&["SELECT * FROM User;", "SELECT * FROM Message;"]).unwrap();
}

// Read each input line and either set our name or send a message
fn user_input_loop() {
    for line in std::io::stdin().lines() {
        let Ok(line) = line else {
            panic!("Failed to read from stdin.");
        };
        if let Some(name) = line.strip_prefix("/name ") {
            set_name(name.to_string());
        } else {
            send_message(line);
        }
    }
}