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

fn main() {
    register_callbacks();
    // connect_to_db();
    // subscribe_to_tables();
    // user_input_loop();
}

//Register all the callbacks our app will use to respond to database events
fn register_callbacks() {
    once_on_connect(on_connected); // Save received credentials to file
    User::on_insert(on_user_inserted); // Notify when new user joins
    User::on_update(on_user_updated); // Notify when a user's status changes
    // Message::on_insert(on_message_inserted); // Notify when a new message is received
    // on_subscription_applied(on_sub_applied); // Print backlog in timestamp order
    // on_set_name(on_name_set); // Print a warning when we fail to set our name
    // on_send_message(on_message_sent); // Print a warning when we fail to send a message
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