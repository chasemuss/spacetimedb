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

}