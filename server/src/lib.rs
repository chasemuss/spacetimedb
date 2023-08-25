use spacetimedb::{
    spacetimedb, // Attribute Macro used to define tables & reducers
    ReducerContext, // Special Argument for reducers
    Identity, // A unique identifier for each connected client
    Timestamp // A point in time as an unsigned 64-bit count of milliseconds since the UNIX Epoch
};

#[spacetimedb(table)]
pub struct User {
    #[primarykey]
    identity: Identity,
    name: Option<String>,
    online: bool,
}

#[spacetimedb(table)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[spacetimedb(reducer)] // Clients invoke this reducer to set their usernames
pub fn set_name(ctx: ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = User::filter_by_identity(&ctx.sender) {
        User::update_by_identity(&ctx.sender, User { name: Some(name), ..user });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

// Take a name and checks to see if it is acceptable as a username
fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}

#[spacetimedb(reducer)] // Clients invoke this reducer to send messages
pub fn send_message(ctx: ReducerContext, text: String) -> Result<(), String> {
    let text = validate_message(text)?;
    log::info!("{}", text);
    Message::insert(Message {
        sender: ctx.sender,
        text,
        sent: ctx.timestamp,
    });
    Ok(())
}

// Take a message and check to see if it is valid
fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Messages must not be empty".to_string())
    } else {
        Ok(text)
    }
}

#[spacetimedb(connect)] // Called when a client connects to the Database
pub fn identity_connected(ctx: ReducerContext) {
    if let Some(user) = User::filter_by_identity(&ctx.sender) {
        // If a user is returned, set the 'online: true', but leave 'name' and 'identity' unchanged
        User::update_by_identity(&ctx.sender, User {online: true, ..user});
    } else {
        // If no user is returned, create a new user identity and set them as online
        User::insert(User {
            name: None,
            identity: ctx.sender,
            online: true,
        }).unwrap();
    }
}

#[spacetimedb(disconnect)] // Called when a client disconnects from the database
pub fn identity_disconnected(ctx: ReducerContext) {
    if let Some(user) = User::filter_by_identity(&ctx.sender) {
        User::update_by_identity(&ctx.sender, User { online: false, ..user });
    } else {
        // This should not happen, so throw error when it does
        log::warn!("Disconnect event for unknown user with identity {:?}", ctx.sender);
    }
}