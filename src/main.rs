//! Welcome to nostr-commander!
//!
//! nostr-commander is a simple terminal-based CLI client of
//! Nostr <https://github.com/nostr-protocol>. It lets you create a
//! Nostr user, subscribe and follow posts of other
//! users and send encrypted, private DMs to your Nostr friends.
//!
//! Please help improve the code and add features  :pray:  :clap:
//!
//! Usage:
//! - run `nostr-commander-rs --help`
//!
//! For more information, see read the README.md
//! <https://github.com/8go/nostr-commander-rs/blob/main/README.md>
//! file.

#![allow(dead_code)] // crate-level allow  // Todo
#![allow(unused_variables)] // Todo
#![allow(unused_imports)] // Todo

use atty::Stream;
use clap::{ColorChoice, CommandFactory, Parser, ValueEnum};

use directories::ProjectDirs;
// use mime::Mime;
use chrono::{Duration, Utc};
// use json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fmt::{self, Debug};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::panic;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use tracing::{debug, enabled, error, info, trace, warn, Level};
use tracing_subscriber;
use update_informer::{registry, Check};
use url::Url;

use bitcoin_hashes::sha256::Hash;

use nostr_sdk::{
    nostr::contact::Contact,
    nostr::event::kind::Kind,
    nostr::event::kind::KindBase,
    nostr::event::tag::TagKind,
    nostr::key::XOnlyPublicKey,
    nostr::key::{FromBech32, KeyError, Keys, ToBech32},
    nostr::message::relay::RelayMessage,
    nostr::message::subscription::SubscriptionFilter,
    nostr::util::time,
    // nostr::util::nips::nip04::Error as Nip04Error,
    nostr::Metadata,
    relay::pool::RelayPoolNotifications,
    subscription::Subscription,
    Client,
    RelayPoolNotifications::ReceivedEvent,
    RelayPoolNotifications::ReceivedMessage,
};

// /// import nostr-sdk Client related code of general kind: create_user, delete_user, etc
// mod client;
// use crate::client::dummy;

/// the version number from Cargo.toml at compile time
const VERSION_O: Option<&str> = option_env!("CARGO_PKG_VERSION");
/// fallback if static compile time value is None
const VERSION: &str = "unknown version";
/// the package name from Cargo.toml at compile time, usually nostr-commander
const PKG_NAME_O: Option<&str> = option_env!("CARGO_PKG_NAME");
/// fallback if static compile time value is None
const PKG_NAME: &str = "nostr-commander";
/// the name of binary program from Cargo.toml at compile time, usually nostr-commander-rs
const BIN_NAME_O: Option<&str> = option_env!("CARGO_BIN_NAME");
/// fallback if static compile time value is None
const BIN_NAME: &str = "nostr-commander-rs";
/// he repo name from Cargo.toml at compile time,
/// e.g. string `https://github.com/8go/nostr-commander-rs/`
const PKG_REPOSITORY_O: Option<&str> = option_env!("CARGO_PKG_REPOSITORY");
/// fallback if static compile time value is None
const PKG_REPOSITORY: &str = "https://github.com/8go/nostr-commander-rs/";
/// default name for login credentials JSON file
const CREDENTIALS_FILE_DEFAULT: &str = "credentials.json";
// /// default timeouts for waiting for the Nostr server, in seconds
// const TIMEOUT_DEFAULT: u64 = 60;
/// default POW difficulty
const POW_DIFFICULTY_DEFAULT: u8 = 20;
/// URL for README.md file downloaded for --readme
const URL_README: &str = "https://raw.githubusercontent.com/8go/nostr-commander-rs/main/README.md";

/// The enumerator for Errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(&'static str),

    #[error("No valid home directory path")]
    NoHomeDirectory,

    #[error("Failure with key")]
    KeyFailure,

    #[error("User Already Exists")]
    UserAlreadyExists,

    #[error("Failure while storing data")]
    StorageFailure,

    #[error("Invalid File")]
    InvalidFile,

    #[error("Creating User Failed")]
    CreatingUserFailed,

    #[error("Reading Credentials Failed")]
    ReadingCredentialsFailed,

    #[error("Cannot Connect To Relays")]
    CannotConnectToRelays,

    #[error("Add Relay Failed")]
    AddRelayFailed,

    #[error("Conversion Failed")]
    ConversionFailed,

    #[error("Publish Failed")]
    PublishFailed,

    #[error("Publish POW Failed")]
    PublishPowFailed,

    #[error("DM Failed")]
    DmFailed,

    #[error("Send Failed")]
    SendFailed,

    #[error("Send Channel Failed")]
    SendChannelFailed,

    #[error("Listen Failed")]
    ListenFailed,

    #[error("Subscription Failed")]
    SubscriptionFailed,

    #[error("Get Entity Failed")]
    GetEntityFailed,

    #[error("Invalid Client Connection")]
    InvalidClientConnection,

    #[error("Invalid Key")]
    InvalidKey,

    #[error("Invalid Hash")]
    InvalidHash,

    #[error("Unknown CLI parameter")]
    UnknownCliParameter,

    #[error("Unsupported CLI parameter")]
    UnsupportedCliParameter,

    #[error("Missing User")]
    MissingUser,

    #[error("Missing Password")]
    MissingPassword,

    #[error("Missing CLI parameter")]
    MissingCliParameter,

    #[error("Not Implemented Yet")]
    NotImplementedYet,

    #[error("No Credentials Found")]
    NoCredentialsFound,

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    NostrNip04(#[from] nostr_sdk::nostr::util::nips::nip04::Error),

    #[error(transparent)]
    NostrKey(#[from] nostr_sdk::nostr::key::KeyError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

/// Function to create custom error messaages on the fly with static text
#[allow(dead_code)]
impl Error {
    pub(crate) fn custom<T>(message: &'static str) -> Result<T, Error> {
        Err(Error::Custom(message))
    }
}

// impl From<anyhow::Error> for Error {
//     fn from(e: anyhow::Error) -> Self {
//         error!("Error: {:?}", e); // full details incl. cause and backtrace
//         Error::Custom(&e.to_string())
//     }
// }

/// Enumerator used for --version option
#[derive(Clone, Debug, Copy, PartialEq, Default, ValueEnum)]
enum Version {
    /// Check if there is a newer version available
    #[default]
    Check,
}

/// is_ functions for the enum
// impl Version {
//     pub fn is_check(&self) -> bool {
//         self == &Self::Check
//     }
// }

/// Converting from String to Version for --version option
impl FromStr for Version {
    type Err = ();
    fn from_str(src: &str) -> Result<Version, ()> {
        return match src.to_lowercase().trim() {
            "check" => Ok(Version::Check),
            _ => Err(()),
        };
    }
}

/// Creates .to_string() for Sync for --sync option
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

/// Enumerator used for --log-level option
#[derive(Clone, Debug, Copy, PartialEq, Default, ValueEnum)]
enum LogLevel {
    /// None: not set, default.
    #[default]
    None,
    /// Error: Indicates to print only errors
    Error,
    /// Warn: Indicates to print warnings and errors
    Warn,
    /// Info: Indicates to to print info, warn and errors
    Info,
    /// Debug: Indicates to to print debug and the rest
    Debug,
    /// Trace: Indicates to to print everything
    Trace,
}

/// is_ functions for the enum
impl LogLevel {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }
    // pub fn is_error(&self) -> bool { self == &Self::Error }
}

/// Creates .to_string() for Listen for --listen option
impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

/// Enumerator used for --output option
#[derive(Clone, Debug, Copy, PartialEq, Default, ValueEnum)]
enum Output {
    // None: only useful if one needs to know if option was used or not.
    // Sort of like an or instead of an Option<Sync>.
    // We do not need to know if user used the option or not,
    // we just need to know the value.
    /// Text: Indicates to print human readable text, default
    #[default]
    Text,
    /// Json: Indicates to print output in Json format
    Json,
    /// Json Max: Indicates to to print the maximum anount of output in Json format
    JsonMax,
    /// Json Spec: Indicates to to print output in Json format, but only data that is according to Nostr Specifications
    JsonSpec,
}

/// is_ functions for the enum
impl Output {
    pub fn is_text(&self) -> bool {
        self == &Self::Text
    }

    // pub fn is_json_spec(&self) -> bool { self == &Self::JsonSpec }
}

/// Converting from String to Listen for --listen option
impl FromStr for Output {
    type Err = ();
    fn from_str(src: &str) -> Result<Output, ()> {
        return match src.to_lowercase().replace('-', "_").trim() {
            "text" => Ok(Output::Text),
            "json" => Ok(Output::Json),
            "jsonmax" | "json_max" => Ok(Output::JsonMax), // accept all 3: jsonmax, json-max, json_max
            "jsonspec" | "json_spec" => Ok(Output::JsonSpec),
            _ => Err(()),
        };
    }
}

/// Creates .to_string() for Listen for --listen option
impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

// A public struct with private fields to keep the command line arguments from
// library `clap`.
/// Welcome to "nostr-commander-rs", a Nostr CLI client. ───
/// On the first run use --create-user to create a user.
/// On further runs you can publish notes, send private DM messages,
/// etc.  ───
/// Have a look at the repo "https://github.com/8go/nostr-commander-rs/"
/// and see if you can contribute code to improve this tool.
/// Safe!
#[derive(Clone, Debug, Parser)]
#[command(author, version,
    next_line_help = true,
    bin_name = get_prog_without_ext(),
    color = ColorChoice::Always,
    term_width = 79,
    after_help = "",
    disable_version_flag = true,
    disable_help_flag = true,
)]
pub struct Args {
    // This is an internal field used to store credentials.
    // The user is not setting this in the CLI.
    // This field is here to simplify argument passing.
    #[arg(skip)]
    creds: Credentials,

    /// Please contribute.
    #[arg(long, default_value_t = false)]
    contribute: bool,

    /// Print version number or check if a newer version exists on crates.io.
    /// If used without an argument such as '--version' it will
    /// print the version number. If 'check' is added ('--version check')
    /// then the program connects to https://crates.io and gets the version
    /// number of latest stable release. There is no "calling home"
    /// on every run, only a "check crates.io" upon request. Your
    /// privacy is protected. New release is neither downloaded,
    /// nor installed. It just informs you.
    #[arg(short, long, value_name = "CHECK")]
    version: Option<Option<Version>>,

    /// Prints very short help summary.
    /// Details:: See also --help, --manual and --readme.
    #[arg(long)]
    usage: bool,

    /// Prints short help.
    /// Details:: See also --usage, --manual and --readme.
    #[arg(short, long)]
    help: bool,

    /// Prints long help.
    /// Details:: See also --usage, --help and --readme.
    #[arg(long)]
    manual: bool,

    /// Prints README.md file, the documenation in Markdown.
    /// Details:: See also --usage, --help and --manual.
    #[arg(long)]
    readme: bool,

    /// Overwrite the default log level. If not used, then the default
    /// log level set with environment variable 'RUST_LOG' will be used.
    /// If used, log level will be set to 'DEBUG' and debugging information
    /// will be printed.
    /// '-d' is a shortcut for '--log-level DEBUG'.
    /// See also '--log-level'. '-d' takes precedence over '--log-level'.
    /// Additionally, have a look also at the option '--verbose'.
    #[arg(short, long,  action = clap::ArgAction::Count, default_value_t = 0u8, )]
    debug: u8,

    /// Set the log level by overwriting the default log level.
    /// If not used, then the default
    /// log level set with environment variable 'RUST_LOG' will be used.
    /// See also '--debug' and '--verbose'.
    // Possible values are
    // '{trace}', '{debug}', '{info}', '{warn}', and '{error}'.
    #[arg(long, value_enum, default_value_t = LogLevel::default(), ignore_case = true, )]
    log_level: LogLevel,

    /// Set the verbosity level. If not used, then verbosity will be
    /// set to low. If used once, verbosity will be high.
    /// If used more than once, verbosity will be very high.
    /// Verbosity only affects the debug information.
    /// So, if '--debug' is not used then '--verbose' will be ignored.
    #[arg(long,  action = clap::ArgAction::Count, default_value_t = 0u8, )]
    verbose: u8,

    // /// Disable encryption for a specific action. By default encryption is
    // /// turned on wherever possible. E.g. rooms created will be created
    // /// by default with encryption enabled. To turn encryption off for a
    // /// specific action use --plain. Currently --plain is supported by
    // /// --room-create and --room-dm-create. See also --room-enable-encryption
    // /// which sort of does the opossite for rooms.
    // #[arg(long, default_value_t = false)]
    // plain: bool,
    /// Path to a file containing credentials.
    /// At --create-user, information about the user, in particular
    /// its keys, will be written to a credentials file. By
    /// default, this file is "credentials.json". On further
    /// runs the credentials file is read to permit acting
    /// as this established Nostr user.
    /// If this option is provided,
    /// the provided path to a file will be used as credentials
    /// file instead of the default one.
    // e.g. /home/user/.local/share/nostr-commander-rs/credentials.json
    #[arg(short, long,
        value_name = "PATH_TO_FILE",
        value_parser = clap::value_parser!(PathBuf),
        default_value_os_t = get_credentials_default_path(),
        )]
    credentials: PathBuf,

    /// Create a new user, i.e. a new key pair. This is usually
    /// done only once at the beginning. If you ever want to wipe
    /// this user, use '--delete-user' which deletes the key
    /// pair. Use this option in combination with --name,
    ///  --display_name, --about, --picture, and --nip05.
    /// Also highly recommended that you use this option
    /// together with --add-relay.
    #[arg(long, alias = "create-key", default_value_t = false)]
    create_user: bool,

    /// Delete the current user, i.e. delete the current key pair.
    /// This will erase the key pair and other associated information
    /// like user name, display name, etc. Afterwards one can create
    /// a new user with '--create-user'.
    #[arg(long, alias = "delete-key", default_value_t = false)]
    delete_user: bool,

    /// Used this to specify an optional user name. Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "USER_NAME")]
    name: Option<String>,

    /// Used this to specify an optional display name. Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "DISPLAY_NAME")]
    display_name: Option<String>,

    /// Used this to specify an optional description. Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "DESCRIPTION")]
    about: Option<String>,

    /// Used this to specify an optional picture or avatar. Used together with
    /// '--create-user'. Provide a URL like 'https://example.com/avatar.png'.
    // or a local file like 'file://somepath/someimage.jpg'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide this URL 'none:'.
    #[arg(long, value_name = "URL")]
    picture: Option<Url>,

    /// Used this to specify an optional nip05 name. Used together with
    /// '--create-user'. Provide a nip05 name like 'john@example.org'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "NIP05_ID")]
    nip05: Option<String>,

    /// Publish one or multiple notes.
    /// Notes data must not be binary data, it
    /// must be text.
    /// Input piped via stdin can additionally be specified with the
    /// special character '-'.
    /// If you want to feed a text message into the program
    /// via a pipe, via stdin, then specify the special
    /// character '-'.
    /// If your message is literally a single letter '-' then use an
    /// escaped '\-' or a quoted "\-".
    /// Depending on your shell, '-' might need to be escaped.
    /// If this is the case for your shell, use the escaped '\-'
    /// instead of '-' and '\\-' instead of '\-'.
    /// However, depending on which shell you are using and if you are
    /// quoting with double quotes or with single quotes, you may have
    /// to add backslashes to achieve the proper escape sequences.
    /// If you want to read the message from
    /// the keyboard use '-' and do not pipe anything into stdin, then
    /// a message will be requested and read from the keyboard.
    /// Keyboard input is limited to one line.
    /// The stdin indicator '-' may appear in any position,
    /// i.e. --publish 'start' '-' 'end'
    /// will send 3 messages out of which the second one is read from stdin.
    /// The stdin indicator '-' may appear only once overall in all arguments.
    /// '-' reads everything that is in the pipe in one swoop and
    /// sends a single message.
    /// Similar to '-', another shortcut character
    /// is '_'. The special character '_' is used for
    /// streaming data via a pipe on stdin. With '_' the stdin
    /// pipe is read line-by-line and each line is treated as
    /// a separate message and sent right away. The program
    /// waits for pipe input until the pipe is closed. E.g.
    /// Imagine a tool that generates output sporadically
    /// 24x7. It can be piped, i.e. streamed, into
    /// nostr-commander, and nostr-commander stays active, sending
    /// all input instantly. If you want to send the literal
    /// letter '_' then escape it and send '\_'. '_' can be
    /// used only once. And either '-' or '_' can be used.
    #[arg(short, long, value_name = "NOTE", num_args(0..), )]
    publish: Vec<String>,

    /// Publish one or multiple notes with proof-of-work (POW).
    /// Use also '--pow-difficulty' to specify difficulty.
    /// See also '--publish' to see how shortcut characters
    /// '-' (pipe) and '_' (streamed pipe) are handled.
    #[arg(long, alias = "pow", value_name = "NOTE", num_args(0..), )]
    publish_pow: Vec<String>,

    /// Send one or multiple DMs to one given user.
    /// DM messages will be encrypted and preserve privacy.
    /// The single recipient is specified via its public key, a
    /// string in the form of 'npub1...', a Hex key, or an alias from
    /// one of your contacts. The first argument
    /// is the recipient, all further arguments are texts to be
    /// sent. E.g. '-dm "npub1SomeStrangeNumbers" "First msg" "Second msg"'
    /// or '--dm joe "How about pizza tonight?"'.
    /// See also '--publish' to see how shortcut characters
    /// '-' (pipe) and '_' (streamed pipe) are handled.
    #[arg(long, alias = "direct", value_name = "KEY+MSGS", num_args(0..), )]
    dm: Vec<String>,

    /// Send one or multiple messages to one given channel.
    /// The single destination channel is specified via its hash.
    /// See here for a channel list: https://damus.io/channels/.
    /// The first argument
    /// is the channel hash, all further arguments are texts to be
    /// sent. E.g.
    /// '-send_channel_message "SomeChannelHash" "First msg" "Second msg"'.
    // or '--send_channel_message joe "How about pizza tonight?"'.
    /// See also '--publish' to see how shortcut characters
    /// '-' (pipe) and '_' (streamed pipe) are handled.
    #[arg(long, alias = "chan", value_name = "HASH+MSGS", num_args(0..), )]
    send_channel_message: Vec<String>,

    /// Add one or multiple relays. A relay is specified via a URI
    /// that looks like 'wss://some.relay.org'. You can find relays
    /// by looking at https://github.com/aljazceru/awesome-nostr#instances.
    /// Sampler relay registries are: https://nostr-registry.netlify.app/,
    /// https://nostr.info/, or https://nostr.watch/.
    /// Examples: "wss://relay.damus.io", "wss://nostr.openchain.fr".
    /// See also '--proxy'.
    #[arg(long, value_name = "RELAY_URI", num_args(0..), )]
    add_relay: Vec<String>,

    // todo remove-relay

    // /// Specify one or multiple tag to attach to notes ot DMs.
    // #[arg(long)]
    // tag: Vec<String>,
    /// Display current metadata.
    #[arg(long, default_value_t = false)]
    show_metadata: bool,

    /// Modify existing metadata of the user.
    /// Use this option in combination with --name,
    ///  --display_name, --about, --picture, and --nip05.
    #[arg(long, default_value_t = false)]
    change_metadata: bool,

    /// Optional proof-of-work (POW) difficulty.
    /// Use with '--publish_pow' to specify difficulty.
    /// If not specified the default will be used.
    #[arg(long, value_name = "DIFFICULTY", default_value_t = POW_DIFFICULTY_DEFAULT, )]
    pow_difficulty: u8,

    /// Specify a proxy. Used by --add-relay.
    #[arg(long)]
    proxy: Option<SocketAddr>,

    /// Show public key.
    #[arg(long, default_value_t = false)]
    show_public_key: bool,

    /// Show private, secret key. Protect this key.
    #[arg(long, default_value_t = false)]
    show_secret_key: bool,

    /// Print the user name used by "nostr-commander-rs".
    /// One can get this information also by looking at the
    /// credentials file or by using --show-metadata.
    #[arg(long)]
    whoami: bool,

    /// This option decides on how the output is presented.
    /// Currently offered choices are: 'text', 'json', 'json-max',
    /// and 'json-spec'. Provide one of these choices.
    /// The default is 'text'. If you want to use the default,
    /// then there is no need to use this option. If you have
    /// chosen 'text', the output will be formatted with the
    /// intention to be consumed by humans, i.e. readable
    /// text. If you have chosen 'json', the output will be
    /// formatted as JSON. The content of the JSON object
    /// matches the data provided by the nostr-sdk SDK. In
    /// some occassions the output is enhanced by having a few
    /// extra data items added for convenience. In most cases
    /// the output will be processed by other programs rather
    /// than read by humans. Option 'json-max' is practically
    /// the same as 'json', but yet another additional field
    /// is added.
    /// In most cases the output will
    /// be processed by other programs rather than read by
    /// humans. Option 'json-spec' only prints information
    /// that adheres 1-to-1 to the Nostr Specification.
    /// Currently this type is not supported.
    /// If no data is available that corresponds exactly with
    /// the Nostr Specification, no data will be printed.
    #[arg(short, long, value_enum,
        value_name = "OUTPUT_FORMAT",
        default_value_t = Output::default(), ignore_case = true, )]
    output: Output,

    /// Listen to events, notifications and messages.
    /// This option listens to events and messages forever. To stop, type
    /// Control-C on your keyboard. You want to listen if you want
    /// to get the event ids for published notices.
    /// Subscriptions do not automatically turn listening on.
    /// If you want to listen to your subscriptions, you must use
    /// --listen.
    #[arg(short, long, default_value_t = false)]
    listen: bool,

    /// Add one or more contacts. Must be used in combination with
    /// --alias, --key, --relay. If you want to add N new contacts,
    /// use --add-contact and provide exactly N entries in each
    /// of the 3 extra arguments. E.g. --add-contact --alias jane joe
    /// --key npub1JanesPublicKey npub1JoesPublicKey
    /// --relay "wss://janes.relay.org" "wss://joes.relay.org".
    /// Aliases must be unique. Alias can be seen as a nickname.
    #[arg(long, default_value_t = false)]
    add_contact: bool,

    /// Remove one or more contacts. Must be used in combination with
    /// --alias. For each entry in --alias the corresponding contact will
    /// be removed. E.g. --remove-contact --alias jane joe.
    #[arg(long, default_value_t = false)]
    remove_contact: bool,

    /// Display current contacts.
    #[arg(long, default_value_t = false)]
    show_contacts: bool,

    /// Provide one or multiple aliases (nicknames) for arguments
    /// --add-contact and --remove-contact.
    #[arg(long, value_name = "ALIAS", num_args(0..), )]
    alias: Vec<String>,

    /// Provide one or multiple public keys for argument
    /// --add-contact. They have the form 'npub1SomeStrangeString'.
    // todo: allow Hex keys
    #[arg(long, value_name = "KEY", num_args(0..), )]
    key: Vec<String>,

    /// Provide one or multiple relays for argument
    /// --add-contact. They have the form 'wss://some.relay.org'.
    #[arg(long, value_name = "RELAY", num_args(0..), )]
    relay: Vec<Url>,

    /// Convert one or multiple public keys in Bech32 format ('npub1...') into
    /// the corresponding 'hex' format.
    /// Details:: See also --hex-to-npub.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    npub_to_hex: Vec<String>,

    /// Convert one or multiple public keys in 'hex' format into
    /// the corresponding Bech32 ('npub1...') format.
    /// Details:: See also --npub-to-hex.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    hex_to_npub: Vec<String>,

    /// Get the entity of one or multiple public keys.
    /// Details:: This will show you
    /// for every public key given if the key represents a Nostr account
    /// (usually an individual) or a public Nostr channel. It might also
    /// return "Unknown" if the entity of the key cannot be determined.
    /// E.g. this can be helpful to determine if you want to use
    /// --subscribe-author or --subscribe-channel.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    get_pubkey_entity: Vec<String>,

    /// Subscribe to one or more public keys.
    /// Details: Specify each
    /// public key in form of 'npub1SomePublicKey'.
    /// Alternatively you can use the Hex form of the public key.
    /// Use this option to subscribe to an account, i.e. the key of
    /// an individual.
    /// See also --subscribe-channel which are different.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    subscribe_pubkey: Vec<String>,

    /// Subscribe to authors with to one or more public keys of accounts.
    /// Details:: Specify each
    /// public key in form of 'npub1SomePublicKey'.
    /// Alternatively you can use the Hex form of the public key.
    /// Use this option to subscribe to a Nostr accounts (usually individuals).
    /// Provide keys that represent accounts (see --get-pubkey-entity).
    /// See also --subscribe-pubkey and --subscribe-channel which are different.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    subscribe_author: Vec<String>,

    /// Subscribe to public channels with one or more public keys of channels.
    /// Details:: Specify each
    /// public key in form of 'npub1SomePublicKey'.
    /// Alternatively you can use the Hex form of the public key.
    /// Sometimes the public key of a public channel is referred to as
    /// channel id.
    /// Provide keys that represent public channels (see --get-pubkey-entity).
    /// See also --subscribe-pubkey and --subscribe-author which are different.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    subscribe_channel: Vec<String>,

    // todo: unsubscribe_pubkey
    // todo unsubscribe_author
    // todo: unsubscribe_channel

    //
    /// Limit the number of messages to receive when subscribing.
    /// By default there is no limit (0).
    #[arg(long, value_name = "NUMBER", default_value_t = 0)]
    limit_number: u16,

    /// Limit the messages received to the last N days when subscribing.
    /// By default there is no limit (0).
    #[arg(long, alias = "since-days", value_name = "DAYS", default_value_t = 0)]
    limit_days: i64,

    /// Limit the messages received to the last N hours when subscribing.
    /// By default there is no limit (0).
    #[arg(long, alias = "since-hours", value_name = "HOURS", default_value_t = 0)]
    limit_hours: i64,

    /// Limit the messages received to the next N days when subscribing.
    /// Stop receiving N days in the future.
    /// By default there is no limit (0).
    #[arg(long, alias = "until-days", value_name = "DAYS", default_value_t = 0)]
    limit_future_days: i64,

    /// Limit the messages received to the last N hours when subscribing.
    /// Stop receiving N hours in the future.
    /// By default there is no limit (0).
    #[arg(long, alias = "until-hours", value_name = "HOURS", default_value_t = 0)]
    limit_future_hours: i64,
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl Args {
    pub fn new() -> Args {
        Args {
            creds: Credentials::new(),
            usage: false,
            help: false,
            manual: false,
            readme: false,
            contribute: false,
            version: None,
            debug: 0u8,
            log_level: LogLevel::None,
            verbose: 0u8,
            // plain: false,
            // credentials file path
            credentials: get_credentials_default_path(),
            create_user: false,
            delete_user: false,
            name: None,
            display_name: None,
            about: None,
            picture: None,
            nip05: None,
            publish: Vec::new(),
            publish_pow: Vec::new(),
            dm: Vec::new(),
            send_channel_message: Vec::new(),
            add_relay: Vec::new(),
            // tag: Vec::new(),
            show_metadata: false,
            change_metadata: false,
            pow_difficulty: POW_DIFFICULTY_DEFAULT,
            proxy: None,
            show_public_key: false,
            show_secret_key: false,
            whoami: false,
            output: Output::default(),
            listen: false,
            add_contact: false,
            remove_contact: false,
            show_contacts: false,
            alias: Vec::new(),
            key: Vec::new(),
            relay: Vec::new(),
            npub_to_hex: Vec::new(),
            hex_to_npub: Vec::new(),
            get_pubkey_entity: Vec::new(),
            subscribe_pubkey: Vec::new(),
            subscribe_author: Vec::new(),
            subscribe_channel: Vec::new(),
            limit_number: 0,
            limit_days: 0,
            limit_hours: 0,
            limit_future_days: 0,
            limit_future_hours: 0,
        }
    }
}

/// A struct for the credentials. These will be serialized into JSON
/// and written to the credentials.json file for permanent storage and
/// future access.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    secret_key_bech32: String, // nsec1...// private key
    public_key_bech32: String, // npub1...
    relays: Vec<Url>,
    metadata: Metadata,
    contacts: Vec<Contact>,
    subscribed_pubkeys: Vec<XOnlyPublicKey>,
    subscribed_authors: Vec<XOnlyPublicKey>,
    subscribed_channels: Vec<XOnlyPublicKey>,
}

impl AsRef<Credentials> for Credentials {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Self::new()
    }
}

/// implementation of Credentials struct
impl Credentials {
    /// Default constructor
    fn new() -> Self {
        Self {
            secret_key_bech32: "".to_owned(),
            public_key_bech32: "".to_owned(),
            relays: Vec::new(),
            metadata: Metadata::new(),
            contacts: Vec::new(),
            subscribed_pubkeys: Vec::new(),
            subscribed_authors: Vec::new(),
            subscribed_channels: Vec::new(),
        }
    }

    // /// Default constructor
    // fn create(
    //     secret_key_bech32: String,
    //     public_key_bech32: String,
    //     relays: Vec<Url>,
    //     metadata: Metadata,
    // ) -> Self {
    //     Self {
    //         secret_key_bech32,
    //         public_key_bech32,
    //         relays,
    //         metadata,
    //     }
    // }

    /// Constructor for Credentials
    fn load(path: &Path) -> Result<Credentials, Error> {
        let reader = File::open(path)?;
        Credentials::set_permissions(&reader)?;
        let credentials: Credentials = serde_json::from_reader(reader)?;
        let mut credentialsfiltered = credentials.clone();
        credentialsfiltered.secret_key_bech32 = "***".to_string();
        info!("loaded credentials are: {:?}", credentialsfiltered);
        Ok(credentials)
    }

    /// Writing the credentials to a file
    fn save(&self, path: &Path) -> Result<(), Error> {
        fs::create_dir_all(path.parent().ok_or(Error::NoHomeDirectory)?)?;
        let writer = File::create(path)?;
        serde_json::to_writer_pretty(&writer, self)?;
        Credentials::set_permissions(&writer)?;
        Ok(())
    }

    #[cfg(unix)]
    fn set_permissions(file: &File) -> Result<(), Error> {
        use std::os::unix::fs::PermissionsExt;
        let perms = file.metadata()?.permissions();
        // is the file world-readable? if so, reset the permissions to 600
        if perms.mode() & 0o4 == 0o4 {
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .unwrap();
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_permissions(file: &File) -> Result<(), Error> {
        Ok(())
    }
}

/// Gets the *default* path (including file name) of the credentials file
/// The default path might not be the actual path as it can be overwritten with command line
/// options.
fn get_credentials_default_path() -> PathBuf {
    let dir = ProjectDirs::from_path(PathBuf::from(get_prog_without_ext())).unwrap();
    // fs::create_dir_all(dir.data_dir());
    let dp = dir.data_dir().join(CREDENTIALS_FILE_DEFAULT);
    debug!(
        "Data will be put into project directory {:?}.",
        dir.data_dir()
    );
    info!("Credentials file with private key is {}.", dp.display());
    dp
}

/// Gets the *actual* path (including file name) of the credentials file
/// The default path might not be the actual path as it can be overwritten with command line
/// options.
fn get_credentials_actual_path(ap: &Args) -> &PathBuf {
    &ap.credentials
}

/// Return true if credentials file exists, false otherwise
fn credentials_exist(ap: &Args) -> bool {
    let dp = get_credentials_default_path();
    let ap = get_credentials_actual_path(ap);
    debug!(
        "credentials_default_path = {:?}, credentials_actual_path = {:?}",
        dp, ap
    );
    let exists = ap.is_file();
    if exists {
        debug!("{:?} exists and is file. Not sure if readable though.", ap);
    } else {
        debug!("{:?} does not exist or is not a file.", ap);
    }
    exists
}

/// Gets version number, static if available, otherwise default.
fn get_version() -> &'static str {
    VERSION_O.unwrap_or(VERSION)
}

/// Gets Rust package name, static if available, otherwise default.
fn get_pkg_name() -> &'static str {
    PKG_NAME_O.unwrap_or(PKG_NAME)
}

/// Gets Rust binary name, static if available, otherwise default.
fn get_bin_name() -> &'static str {
    BIN_NAME_O.unwrap_or(BIN_NAME)
}

/// Gets Rust package repository, static if available, otherwise default.
fn get_pkg_repository() -> &'static str {
    PKG_REPOSITORY_O.unwrap_or(PKG_REPOSITORY)
}

/// Gets program name without extension.
fn get_prog_without_ext() -> &'static str {
    get_bin_name() // with -rs suffix
                   // get_pkg_name() // without -rs suffix
}

/// Prints the usage info
pub fn usage() {
    let help_str = Args::command().render_usage().to_string();
    println!("{}", &help_str);
}

/// Prints the short help
pub fn help() {
    let help_str = Args::command().render_help().to_string();
    println!("{}", &help_str);
}

/// Prints the long help
pub fn manual() {
    let help_str = Args::command().render_long_help().to_string();
    println!("{}", &help_str);
}

/// Prints the README.md file
pub async fn readme() {
    match reqwest::get(URL_README).await {
        Ok(resp) => {
            debug!("Got README.md file from URL {:?}.", URL_README);
            println!("{}", resp.text().await.unwrap())
        }
        Err(ref e) => {
            println!(
                "Error getting README.md from {:#?}. Reported error {:?}.",
                URL_README, e
            );
        }
    };
}

/// Prints the version information
pub fn version() {
    println!();
    println!(
        "  _|      _|      _|_|_|                     {}",
        get_prog_without_ext()
    );
    print!("  _|_|    _|    _|             _~^~^~_       ");
    println!("a Nostr CLI client written in Rust");
    println!(
        "  _|  _|  _|    _|         \\) /  o o  \\ (/   version {}",
        get_version()
    );
    println!(
        "  _|    _| |    _|           '_   -   _'     repo {}",
        get_pkg_repository()
    );
    print!("  _|      _|      _|_|_|     / '-----' \\     ");
    println!("please submit PRs to make this a better tool");
    println!();
}

/// Prints the installed version and the latest crates.io-available version
pub fn version_check() {
    println!("Installed version: v{}", get_version());
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let informer = update_informer::new(registry::Crates, name, version).check_version();
    match informer {
        Ok(Some(version)) => println!(
            "New version is available on https://crates.io/crates/{}: {}",
            name, version
        ),
        Ok(None) => println!("You are up-to-date. You already have the latest version."),
        Err(ref e) => println!("Could not get latest version. Error reported: {:?}.", e),
    };
}

/// Asks the public for help
pub fn contribute() {
    println!();
    println!("This project is currently an experiment. ",);
    println!("If you know Rust and are interested in Nostr, please have a look at the repo ");
    println!("{}. ", get_pkg_repository());
    println!(
        "Please contribute code to improve the {} ",
        get_prog_without_ext()
    );
    println!("Nostr CLI client. Safe!");
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_name(ap: &mut Args) {
    print!("Enter an optional name for this Nostr account (e.g. John Doe): ");
    std::io::stdout()
        .flush()
        .expect("error: could not flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    match input.trim() {
        "" => {
            info!("Name left empty. That is okay!");
            ap.creds.metadata.name = None;
        }
        _ => {
            ap.creds.metadata.name = Some(input.trim().to_owned());
            info!("Name set to {:?}.", ap.creds.metadata.name);
        }
    }
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_display_name(ap: &mut Args) {
    print!("Enter an optional display name for this Nostr account (e.g. Jonnie): ");
    std::io::stdout()
        .flush()
        .expect("error: could not flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    match input.trim() {
        "" => {
            info!("Display name left empty. That is okay!");
            ap.creds.metadata.display_name = None;
        }
        _ => {
            ap.creds.metadata.display_name = Some(input.trim().to_owned());
            info!("Display_name set to {:?}.", ap.creds.metadata.display_name);
        }
    }
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_about(ap: &mut Args) {
    print!(
        "Enter an optional description for this Nostr account (e.g. nostr loving surfing dude): "
    );
    std::io::stdout()
        .flush()
        .expect("error: could not flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    match input.trim() {
        "" => {
            info!("About left empty. That is okay!");
            ap.creds.metadata.about = None;
        }
        _ => {
            ap.creds.metadata.about = Some(input.trim().to_owned());
            info!("About set to {:?}.", ap.creds.metadata.about);
        }
    }
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_picture(ap: &mut Args) {
    let mut repeat = true;
    while repeat {
        repeat = false;
        print!("Enter an optional picture for this Nostr account (e.g. 'https://example.com/avatar.png' or 'file://./somedir/localfile.png'): ");
        std::io::stdout()
            .flush()
            .expect("error: could not flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        match input.trim() {
            "" => {
                info!("Picture left empty. That is okay!");
                ap.creds.metadata.picture = None;
            }
            _ => match Url::parse(input.trim()) {
                Ok(_) => {
                    ap.creds.metadata.picture = Some(input.trim().to_owned());
                    info!("Picture set to {:?}.", ap.creds.metadata.picture);
                }
                Err(ref e) => {
                    error!(
                        "{:?} is not a valid URL. Try again or leave empty. Reported error is {:?}.",
                        input.trim(), e
                    );
                    repeat = true;
                }
            },
        }
    }
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_nip05(ap: &mut Args) {
    print!("Enter an optional nip05 name for this Nostr account (e.g. john@example.com): ");
    std::io::stdout()
        .flush()
        .expect("error: could not flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    match input.trim() {
        "" => {
            info!("Nip05 left empty. That is okay!");
            ap.creds.metadata.nip05 = None;
        }
        _ => {
            ap.creds.metadata.nip05 = Some(input.trim().to_owned());
            info!("Nip05 set to {:?}.", ap.creds.metadata.about);
        }
    }
}

/// Reads metadata item from keyboard and puts it into the Args.
fn get_relays(ap: &mut Args) {
    println!("Enter one or multiple optional relays for this Nostr account.");
    let mut repeat = true;
    while repeat {
        print!("Enter relay name (e.g. wss://relay.example.com) or leave empty to move on: ");
        std::io::stdout()
            .flush()
            .expect("error: could not flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        match input.trim() {
            "" => {
                info!("Realay left empty. That is okay!");
                repeat = false;
            }
            _ => match Url::parse(input.trim()) {
                Ok(u) => {
                    if u.scheme() == "wss" {
                        ap.creds.relays.push(u.clone());
                        info!("relay {:?} added.", &u);
                    } else {
                        error!(
                        "{:?} is not a valid URL. Scheme is not 'wss'. Try again or leave empty.",
                        input.trim(),
                    );
                    }
                }
                Err(ref e) => {
                    error!(
                        "{:?} is not a valid URL. Try again or leave empty. Reported error is {:?}.",
                        input.trim(), e
                    );
                }
            },
        }
    }
}

/// Read credentials from disk
pub(crate) fn read_credentials(ap: &mut Args) -> Result<(), Error> {
    match Credentials::load(get_credentials_actual_path(&ap)) {
        Ok(c) => {
            info!(
                "Successfully loaded credentials from credentials file {:?}.",
                get_credentials_actual_path(&ap)
            );
            ap.creds = c;
            return Ok(());
        }
        Err(ref e) => {
            error!(
                "Error: failed to read credentials file {:?}. Aborting. Correct path? Error reported: {:?}.",
                get_credentials_actual_path(&ap),
                e,
            );
            return Err(Error::StorageFailure);
        }
    }
}

/// is this syntactically a valid relay string?
pub(crate) fn is_relay_str(relay: &str) -> bool {
    match Url::parse(relay) {
        Ok(r) => {
            if r.scheme() == "wss" {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// is this syntactically a valid relay string?
pub(crate) fn is_relay_url(relay: &Url) -> bool {
    if relay.scheme() != "wss" {
        return false;
    } else if relay.host_str().is_none() {
        return false;
    }
    true
}

/// Handle the --create_user CLI argument
pub(crate) fn cli_create_user(ap: &mut Args) -> Result<(), Error> {
    if !ap.create_user {
        return Err(Error::UnsupportedCliParameter);
    }
    if credentials_exist(ap) {
        error!(concat!(
            "Credentials file already exists. You have already created a user in ",
            "the past. No --create-user is needed. Aborting. If you really want to create ",
            "a new user, then delete the current user first with '--delete-user', or move ",
            "or remove credentials file manually. ",
            "Or just run your command again but without the '--create-user' option to ",
            "use the currently existing user. ",
        ));
        return Err(Error::UserAlreadyExists);
    }
    match ap.name.as_ref() {
        None => {
            get_name(ap); // read from kb, put into metadata
        }
        Some(n) => {
            if n.trim().is_empty() {
                ap.creds.metadata.name = None;
            } else {
                ap.creds.metadata.name = Some(n.trim().to_owned());
            }
        }
    }
    match ap.display_name.as_ref() {
        None => {
            get_display_name(ap); // read from kb, put into metadata
        }
        Some(n) => {
            if n.trim().is_empty() {
                ap.creds.metadata.display_name = None;
            } else {
                ap.creds.metadata.display_name = Some(n.trim().to_owned());
            }
        }
    }
    match ap.about.as_ref() {
        None => {
            get_about(ap); // read from kb, put into metadata
        }
        Some(n) => {
            if n.trim().is_empty() {
                ap.creds.metadata.about = None;
            } else {
                ap.creds.metadata.about = Some(n.trim().to_owned());
            }
        }
    }
    match ap.picture.as_ref() {
        None => {
            get_picture(ap); // read from kb, put into metadata
        }
        Some(n) => {
            if (n.scheme() == "none" || n.scheme() == "file")
                && (n.path() == "/" || n.path() == "")
                && n.host().is_none()
            {
                ap.creds.metadata.picture = None;
            } else {
                ap.creds.metadata.picture = Some(n.to_string());
            }
        }
    }
    match ap.nip05.as_ref() {
        None => {
            get_nip05(ap); // read from kb, put into metadata
        }
        Some(n) => {
            if n.trim().is_empty() {
                ap.creds.metadata.nip05 = None;
            } else {
                ap.creds.metadata.nip05 = Some(n.trim().to_owned());
            }
        }
    }
    info!("Metadata is: {:?}", ap.creds.metadata);

    if ap.add_relay.is_empty() {
        get_relays(ap);
    } else {
        let num = ap.add_relay.len();
        let mut i = 0;
        while i < num {
            if is_relay_str(&ap.add_relay[i]) {
                ap.creds.relays.push(Url::parse(&ap.add_relay[i]).unwrap());
            } else {
                error!(
                    "Invalid relay syntax for relay {:?}. Skipping it.",
                    ap.add_relay[i]
                )
            }
            i += 1;
        }
    }
    ap.creds.relays.dedup_by(|a, b| a == b);

    // Generate new keys
    let my_keys: Keys = Client::generate_keys();
    debug!(
        "Generated private key is: {:?}",
        my_keys.secret_key_as_str()?
    );
    debug!(
        "Generated public  key is: {:?}",
        my_keys.public_key_as_str()
    );
    match my_keys.public_key().to_bech32() {
        Ok(k) => ap.creds.public_key_bech32 = k,
        Err(ref e) => {
            error!(
                "Error: failed to convert public key. Aborting. Error reported: {:?}. ({:?})",
                e, my_keys
            );
            return Err(Error::KeyFailure);
        }
    }
    match my_keys.secret_key()?.to_bech32() {
        Ok(k) => ap.creds.secret_key_bech32 = k,
        Err(ref e) => {
            error!(
                "Error: failed to convert private key. Aborting. Error reported: {:?}. ({:?})",
                e, my_keys
            );
            return Err(Error::KeyFailure);
        }
    }
    match ap.creds.save(get_credentials_actual_path(&ap)) {
        Ok(()) => {
            info!("Successfully stored credentials in credentials file {:?}. Protect it, it contains your private key. Data stored is {:?}.", get_credentials_actual_path(&ap), &ap.creds);
        }
        Err(ref e) => {
            error!(
                "Error: failed to store credentials. Aborting. Error reported: {:?}. ({:?})",
                e, my_keys
            );
            return Err(Error::StorageFailure);
        }
    }
    Ok(())
}

/// Add relays to from Credentials to client
pub(crate) fn add_relays_from_creds(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0u32;
    let num = ap.creds.relays.len();
    let mut i = 0;
    while i < num {
        match client.add_relay(ap.creds.relays[i].as_str(), None) {
            Ok(()) => {
                debug!("add_relay with relay {:?} successful.", ap.creds.relays[i]);
            }
            Err(ref e) => {
                error!(
                    "Error: add_relay() returned error. Relay {:?} not added. Reported error {:?}.",
                    ap.creds.relays[i], e
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::AddRelayFailed)
    } else {
        Ok(())
    }
}

/// Handle the --add_relay CLI argument.
/// Add relays from --add-relay.
pub(crate) fn cli_add_relay(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0u32;
    let num = ap.add_relay.len();
    let mut i = 0;
    while i < num {
        if is_relay_str(ap.add_relay[i].as_str()) {
            match client.add_relay(ap.add_relay[i].as_str(), None) {
                Ok(()) => {
                    debug!("add_relay with relay {:?} successful.", ap.add_relay[i]);
                    ap.creds
                        .relays
                        .push(Url::parse(ap.add_relay[i].as_str()).unwrap());
                }
                Err(ref e) => {
                    error!(
                    "Error: add_relay() returned error. Relay {:?} not added. Reported error {:?}.",
                    ap.add_relay[i], e
                );
                    err_count += 1;
                }
            }
        } else {
            error!(
                "Error: Relay {:?} is syntactically not correct. Relay not added.",
                ap.add_relay[i],
            );
            err_count += 1;
        }
        i += 1;
    }
    ap.creds.relays.dedup_by(|a, b| a == b);
    match ap.creds.save(get_credentials_actual_path(ap)) {
        Ok(()) => {
            debug!(
                "writing new relays {:?} to credentials file successful.",
                ap.creds.relays
            );
        }
        Err(ref e) => {
            error!(
                "Error: writing new relays {:?} to credentials file failed. Reported error {:?}.",
                ap.creds.relays, e
            );
            err_count += 1;
        }
    }
    if err_count != 0 {
        Err(Error::AddRelayFailed)
    } else {
        Ok(())
    }
}

fn trim_newline(s: &mut String) -> &mut String {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    return s;
}

/// Handle the --publish CLI argument
/// Publish notes.
pub(crate) async fn cli_publish(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.publish.len();
    let mut i = 0;
    while i < num {
        let note = &ap.publish[i];
        trace!("publish: {:?}", note);
        if note.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        };
        if note == "--" {
            info!("Skipping '--' text note as these are used to separate arguments.");
            i += 1;
            continue;
        };
        // - map to - (stdin pipe)
        // \- maps to text r'-', a 1-letter message
        let fnote = if note == r"-" {
            let mut line = String::new();
            if atty::is(Stream::Stdin) {
                print!("Message: ");
                std::io::stdout()
                    .flush()
                    .expect("error: could not flush stdout");
                io::stdin().read_line(&mut line)?;
            } else {
                io::stdin().read_to_string(&mut line)?;
            }
            line
        } else if note == r"_" {
            let mut eof = false;
            while !eof {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    // If this function returns Ok(0), the stream has reached EOF.
                    Ok(n) => {
                        if n == 0 {
                            eof = true;
                            debug!("Reached EOF of pipe stream.");
                        } else {
                            debug!(
                                "Read {n} bytes containing \"{}\\n\" from pipe stream.",
                                trim_newline(&mut line.clone())
                            );
                            match client.publish_text_note(&line, &[]).await {
                                Ok(()) => debug!(
                                    "Publish_text_note number {:?} from pipe stream sent successfully. {:?}",
                                    i, &line
                                ),
                                Err(ref e) => {
                                    err_count += 1;
                                    error!(
                                        "Publish_text_note number {:?} from pipe stream failed. {:?}",
                                        i, &line
                                    );
                                }
                            }
                        }
                    }
                    Err(ref e) => {
                        err_count += 1;
                        error!("Error: reading from pipe stream reported {}", e);
                    }
                }
            }
            "".to_owned()
        } else if note == r"\-" {
            "-".to_string()
        } else if note == r"\_" {
            "_".to_string()
        } else if note == r"\-\-" {
            "--".to_string()
        } else if note == r"\-\-\-" {
            "---".to_string()
        } else {
            note.to_string()
        };
        if fnote.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        }

        match client.publish_text_note(&fnote, &[]).await {
            Ok(()) => debug!(
                "Publish_text_note number {:?} sent successfully. {:?}",
                i, &fnote
            ),
            Err(ref e) => {
                err_count += 1;
                error!("Publish_text_note number {:?} failed. {:?}", i, &fnote);
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::PublishFailed)
    } else {
        Ok(())
    }
}

/// Handle the --publish_pow CLI argument
/// Publish a POW text note
pub(crate) async fn cli_publish_pow(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.publish_pow.len();
    let mut i = 0;
    while i < num {
        let note = &ap.publish_pow[i];
        trace!("publish: {:?}", note);
        if note.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        };
        if note == "--" {
            info!("Skipping '--' text note as these are used to separate arguments.");
            i += 1;
            continue;
        };
        // - map to - (stdin pipe)
        // \- maps to text r'-', a 1-letter message
        let fnote = if note == r"-" {
            let mut line = String::new();
            if atty::is(Stream::Stdin) {
                print!("Message: ");
                std::io::stdout()
                    .flush()
                    .expect("error: could not flush stdout");
                io::stdin().read_line(&mut line)?;
            } else {
                io::stdin().read_to_string(&mut line)?;
            }
            line
        } else if note == r"_" {
            let mut eof = false;
            while !eof {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    // If this function returns Ok(0), the stream has reached EOF.
                    Ok(n) => {
                        if n == 0 {
                            eof = true;
                            debug!("Reached EOF of pipe stream.");
                        } else {
                            debug!(
                                "Read {n} bytes containing \"{}\\n\" from pipe stream.",
                                trim_newline(&mut line.clone())
                            );
                            debug!("Be patient, hashing ...");
                            match client.publish_pow_text_note(&line, &[], ap.pow_difficulty).await {
                                Ok(()) => debug!(
                                    "Publish_pow_text_note number {:?} from pipe stream sent successfully. {:?}",
                                    i, &line
                                ),
                                Err(ref e) => {
                                    err_count += 1;
                                    error!(
                                        "Publish_pow_text_note number {:?} from pipe stream failed. {:?}",
                                        i, &line
                                    );
                                }
                            }
                        }
                    }
                    Err(ref e) => {
                        err_count += 1;
                        error!("Error: reading from pipe stream reported {}", e);
                    }
                }
            }
            "".to_owned()
        } else if note == r"\-" {
            "-".to_string()
        } else if note == r"\_" {
            "_".to_string()
        } else if note == r"\-\-" {
            "--".to_string()
        } else if note == r"\-\-\-" {
            "---".to_string()
        } else {
            note.to_string()
        };
        if fnote.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        }

        debug!("Be patient, hashing ...");
        match client
            .publish_pow_text_note(&fnote, &[], ap.pow_difficulty)
            .await
        {
            Ok(()) => debug!(
                "Publish_pow_text_note number {:?} sent successfully. {:?}",
                i, &fnote
            ),
            Err(ref e) => {
                err_count += 1;
                error!("Publish_pow_text_note number {:?} failed. {:?}", i, &fnote);
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::PublishPowFailed)
    } else {
        Ok(())
    }
}

/// Publish DMs.
pub(crate) async fn send_dms(
    client: &Client,
    notes: &[String],
    recipient: &Keys,
) -> Result<(), Error> {
    trace!("send_dms: {:?} {:?}", notes, recipient);
    let mut err_count = 0usize;
    let num = notes.len();
    let mut i = 0;
    while i < num {
        let note = &notes[i];
        trace!("send_dms: {:?}", note);
        if note.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        };
        if note == "--" {
            info!("Skipping '--' text note as these are used to separate arguments.");
            i += 1;
            continue;
        };
        // - map to - (stdin pipe)
        // \- maps to text r'-', a 1-letter message
        let fnote = if note == r"-" {
            let mut line = String::new();
            if atty::is(Stream::Stdin) {
                print!("Message: ");
                std::io::stdout()
                    .flush()
                    .expect("error: could not flush stdout");
                io::stdin().read_line(&mut line)?;
            } else {
                io::stdin().read_to_string(&mut line)?;
            }
            line
        } else if note == r"_" {
            let mut eof = false;
            while !eof {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    // If this function returns Ok(0), the stream has reached EOF.
                    Ok(n) => {
                        if n == 0 {
                            eof = true;
                            debug!("Reached EOF of pipe stream.");
                        } else {
                            debug!(
                                "Read {n} bytes containing \"{}\\n\" from pipe stream.",
                                trim_newline(&mut line.clone())
                            );
                            match client.send_direct_msg(recipient, &line).await {
                                Ok(()) => debug!(
                                    "send_direct_msg number {:?} from pipe stream sent successfully. {:?}, sent to {:?}",
                                    i, &line, recipient.public_key()
                                ),
                                Err(ref e) => {
                                    err_count += 1;
                                    error!(
                                        "send_direct_msg number {:?} from pipe stream failed. {:?}, sent to {:?}",
                                        i, &line, recipient.public_key()
                                    );
                                }
                            }
                        }
                    }
                    Err(ref e) => {
                        err_count += 1;
                        error!("Error: reading from pipe stream reported {}", e);
                    }
                }
            }
            "".to_owned()
        } else if note == r"\-" {
            "-".to_string()
        } else if note == r"\_" {
            "_".to_string()
        } else if note == r"\-\-" {
            "--".to_string()
        } else if note == r"\-\-\-" {
            "---".to_string()
        } else {
            note.to_string()
        };
        if fnote.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        }

        match client.send_direct_msg(recipient, &fnote).await {
            Ok(()) => debug!(
                "DM message number {:?} sent successfully. {:?}, sent to {:?}.",
                i,
                &fnote,
                recipient.public_key()
            ),
            Err(ref e) => {
                err_count += 1;
                error!(
                    "DM message number {:?} failed. {:?}, sent to {:?}.",
                    i,
                    &fnote,
                    recipient.public_key()
                );
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::PublishPowFailed)
    } else {
        Ok(())
    }
}

/// Handle the --dm CLI argument
/// Publish DMs.
pub(crate) async fn cli_dm(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let num = ap.dm.len();
    if num < 2 {
        return Err(Error::MissingCliParameter);
    }
    match cstr_to_pubkey(ap, ap.dm[0].trim()) {
        Ok(pk) => {
            let keys = Keys::from_public_key(pk);
            let notes = &ap.dm[1..];
            send_dms(client, notes, &keys).await
        }
        Err(ref e) => {
            error!(
                "Error: Not a valid key. Cannot send this DM. Aborting. Key {:?}, 1st Msg {:?} ",
                ap.dm[0].trim(),
                ap.dm[1]
            );
            Err(Error::InvalidKey)
        }
    }
}

/// Send messages to one channel.
pub(crate) async fn send_channel_messages(
    client: &Client,
    notes: &[String], // msgs
    channel_id: Hash,
    relay_url: Url,
) -> Result<(), Error> {
    trace!("send_channel_messages {:?} {:?}.", notes, channel_id);
    let mut err_count = 0usize;
    let num = notes.len();
    let mut i = 0;
    while i < num {
        let note = &notes[i];
        trace!("send_dms: {:?}", note);
        if note.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        };
        if note == "--" {
            info!("Skipping '--' text note as these are used to separate arguments.");
            i += 1;
            continue;
        };
        // - map to - (stdin pipe)
        // \- maps to text r'-', a 1-letter message
        let fnote = if note == r"-" {
            let mut line = String::new();
            if atty::is(Stream::Stdin) {
                print!("Message: ");
                std::io::stdout()
                    .flush()
                    .expect("error: could not flush stdout");
                io::stdin().read_line(&mut line)?;
            } else {
                io::stdin().read_to_string(&mut line)?;
            }
            line
        } else if note == r"_" {
            let mut eof = false;
            while !eof {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    // If this function returns Ok(0), the stream has reached EOF.
                    Ok(n) => {
                        if n == 0 {
                            eof = true;
                            debug!("Reached EOF of pipe stream.");
                        } else {
                            debug!(
                                "Read {n} bytes containing \"{}\\n\" from pipe stream.",
                                trim_newline(&mut line.clone())
                            );
                            match client.send_channel_msg(channel_id, relay_url.clone(), &line).await {
                                Ok(()) => debug!(
                                    "send_channel_msg number {:?} from pipe stream sent successfully. {:?}, sent to {:?}",
                                    i, &line, channel_id
                                ),
                                Err(ref e) => {
                                    err_count += 1;
                                    error!(
                                        "send_channel_msg number {:?} from pipe stream failed. {:?}, sent to {:?}",
                                        i, &line, channel_id
                                    );
                                }
                            }
                        }
                    }
                    Err(ref e) => {
                        err_count += 1;
                        error!("Error: reading from pipe stream reported {}", e);
                    }
                }
            }
            "".to_owned()
        } else if note == r"\-" {
            "-".to_string()
        } else if note == r"\_" {
            "_".to_string()
        } else if note == r"\-\-" {
            "--".to_string()
        } else if note == r"\-\-\-" {
            "---".to_string()
        } else {
            note.to_string()
        };
        if fnote.is_empty() {
            info!("Skipping empty text note.");
            i += 1;
            continue;
        }

        match client
            .send_channel_msg(channel_id, relay_url.clone(), &fnote)
            .await
        {
            Ok(()) => debug!(
                "send_channel_msg message number {:?} sent successfully. {:?}, sent to {:?}.",
                i, &fnote, channel_id
            ),
            Err(ref e) => {
                err_count += 1;
                error!(
                    "send_channel_msg message number {:?} failed. {:?}, sent to {:?}.",
                    i, &fnote, channel_id
                );
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::SendChannelFailed)
    } else {
        Ok(())
    }
}

/// Handle the --send-channel-message CLI argument
/// Publish messages to one channel.
pub(crate) async fn cli_send_channel_message(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let num = ap.send_channel_message.len();
    if num < 2 {
        return Err(Error::MissingCliParameter);
    }
    // todo: check if hash is valid, doable?
    match Hash::from_str(&ap.send_channel_message[0]) {
        Ok(hash) => {
            let notes = &ap.send_channel_message[1..];
            // todo: any relay is fine?
            let relay_url: Url = ap.creds.relays[0].clone();
            send_channel_messages(client, notes, hash, relay_url).await
        }
        Err(ref e) => {
            error!(
                "Error: Not a valid hash (channel id). Cannot send this channel message. Aborting. Hash {:?}, 1st Msg {:?} ",
                ap.send_channel_message[0],
                ap.send_channel_message[1]
            );
            Err(Error::InvalidHash)
        }
    }
}

/// Is key in subscribed_authors list?
pub(crate) fn is_subscribed_author(ap: &Args, pkey: &XOnlyPublicKey) -> bool {
    ap.creds.subscribed_authors.contains(pkey)
}

/// Get contact for given alias.
/// Returns None if alias does not exist in contact list.
pub(crate) fn get_contact_by_alias(ap: &Args, alias: &str) -> Option<Contact> {
    ap.creds.contacts.iter().find(|s| s.alias == alias).cloned()
}

/// Get contact for given pubkey.
/// Returns None if pubkey does not exist in contact list.
pub(crate) fn get_contact_by_key(ap: &Args, pkey: XOnlyPublicKey) -> Option<Contact> {
    ap.creds.contacts.iter().find(|s| s.pk == pkey).cloned()
}

/// Get contact alias for given pubkey, or if not in contacts return given pubkey.
/// Returns alias if contact with this pubkey exists.
/// Returns input pubkey if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_or_keystr_by_key(ap: &Args, pkey: XOnlyPublicKey) -> String {
    match get_contact_by_key(ap, pkey) {
        Some(c) => c.alias,
        None => pkey.to_string(),
    }
}

/// Get contact alias for given pubkey, or if not in contacts return None.
/// Returns Some(alias) if contact with this pubkey exists.
/// Returns None if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_by_key(ap: &Args, pkey: XOnlyPublicKey) -> Option<String> {
    match get_contact_by_key(ap, pkey) {
        Some(c) => Some(c.alias),
        None => None,
    }
}

/// Get contact alias for given pubkey string (string of XOnlyPublicKey), or if not in contacts return given pubkey.
/// Returns alias if contact with this pubkey exists.
/// Returns input pubkey if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_or_keystr_by_keystr(ap: &Args, pkeystr: &str) -> String {
    match XOnlyPublicKey::from_str(pkeystr) {
        Ok(pkey) => match get_contact_by_key(ap, pkey) {
            Some(c) => c.alias,
            None => pkey.to_string(),
        },
        Err(_) => pkeystr.to_string(),
    }
}

/// Get contact alias for given pubkey string (string of XOnlyPublicKey), or if not in contacts return None.
/// Returns Some(alias) if contact with this pubkey exists.
/// Returns None if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_by_keystr(ap: &Args, pkeystr: &str) -> Option<String> {
    match XOnlyPublicKey::from_str(pkeystr) {
        Ok(pkey) => match get_contact_by_key(ap, pkey) {
            Some(c) => Some(c.alias),
            None => None,
        },
        Err(_) => None,
    }
}

/// Handle the --add-conect CLI argument, write contacts from CLI args into creds data structure
pub(crate) async fn cli_add_contact(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let anum = ap.alias.len();
    let knum = ap.key.len();
    let rnum = ap.relay.len();
    if (anum != knum) || (anum != rnum) || (knum != rnum) {
        error!(
            "--alias, --key, and --relay must have the same amount of entries. {:?} {:?} {:?} ",
            anum, knum, rnum
        );
        return Err(Error::MissingCliParameter);
    }
    let mut i = 0;
    while i < anum {
        if ap.alias[i].trim().is_empty() {
            error!("Invalid user alias. Cannot be empty. Skipping this contact.");
            err_count += 1;
            i += 1;
            continue;
        }
        if get_contact_by_alias(ap, ap.alias[i].trim()).is_some() {
            error!("Invalid user alias. Alias already exists. Alias must be unique. Skipping this contact.");
            err_count += 1;
            i += 1;
            continue;
        }
        if !is_relay_url(&ap.relay[i]) {
            error!(
                "Relay {:?} is not valid. Skipping this contact.",
                ap.relay[i]
            );
            err_count += 1;
            i += 1;
            continue;
        }
        let key = &ap.key[i];
        match str_to_pubkey(key) {
            Ok(pkey) => {
                debug!("Valid key for contact. Key {:?}, {:?}.", key, pkey);
                ap.creds.contacts.push(Contact::new(
                    pkey,
                    ap.relay[i].to_string().clone(),
                    ap.alias[i].trim().to_string(),
                ));
                debug!("Added contact. Key {:?}, {:?}.", key, pkey);
            }
            Err(ref e) => {
                error!("Error: Invalid key {:?}. Skipping this contact.", key);
                err_count += 1;
                i += 1;
                continue;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::SubscriptionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --add-conect CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_remove_contact(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let num = ap.alias.len();
    let mut i = 0;
    while i < num {
        ap.creds.contacts.retain(|x| x.alias != ap.alias[i].trim());
        i += 1;
    }
    Ok(())
}

/// Convert npub1... Bech32 key or Hex key or contact alias into a XOnlyPublicKey
/// Returns Error if neither valid Bech32, nor Hex key, nor contact alias.
pub(crate) fn cstr_to_pubkey(ap: &Args, s: &str) -> Result<XOnlyPublicKey, Error> {
    match get_contact_by_alias(ap, s) {
        Some(c) => Ok(c.pk),
        None => str_to_pubkey(s),
    }
}

/// Convert npub1... Bech32 key or Hex key into a XOnlyPublicKey
/// Returns Error if neither valid Bech32 nor Hex key.
pub(crate) fn str_to_pubkey(s: &str) -> Result<XOnlyPublicKey, Error> {
    match Keys::from_bech32_public_key(s) {
        Ok(keys) => {
            debug!(
                "Valid key in Bech32 format: Npub {:?}, Hex {:?}",
                s,
                keys.public_key().to_string()
            );
            return Ok(keys.public_key());
        }
        Err(ref e) => match XOnlyPublicKey::from_str(s) {
            Ok(pkey) => {
                debug!(
                    "Valid key in Hex format: Hex {:?}, Npub {:?}",
                    s,
                    pkey.to_bech32().unwrap()
                );
                return Ok(pkey);
            }
            Err(ref e) => {
                error!("Error: Invalid key {:?}. Reported error: {:?}.", s, e);
                return Err(Error::InvalidKey);
            }
        },
    }
}

/// Convert npub1... Bech32 key or Hex key into a npub+hex pair as Vector.
/// s ... input, npub ... output, hex ... output.
/// Returns Error if neither valid Bech32 nor Hex key.
pub(crate) fn str_to_pubkeys(s: &str) -> Result<(String, String), Error> {
    match Keys::from_bech32_public_key(s) {
        Ok(keys) => {
            debug!(
                "Valid key in Bech32 format: Npub {:?}, Hex {:?}",
                s,
                keys.public_key().to_string()
            );
            let npub = s.to_owned();
            let hex = keys.public_key().to_string();
            return Ok((npub, hex));
        }
        Err(ref e) => match XOnlyPublicKey::from_str(s) {
            Ok(pkey) => {
                debug!(
                    "Valid key in Hex format: Hex {:?}, Npub {:?}",
                    s,
                    pkey.to_bech32().unwrap()
                );
                let npub = pkey.to_bech32().unwrap();
                let hex = s.to_owned();
                return Ok((npub, hex));
            }
            Err(ref e) => {
                error!("Error: Invalid key {:?}. Reported error: {:?}.", s, e);
                return Err(Error::InvalidKey);
            }
        },
    }
}

/// Handle the --cli_npub_to_hex CLI argument
pub(crate) fn cli_npub_to_hex(ap: &Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.npub_to_hex.len();
    let mut i = 0;
    while i < num {
        match str_to_pubkeys(&ap.npub_to_hex[i]) {
            Ok((npub, hex)) => {
                debug!("Valid key. Npub {:?}, Hex: {:?}.", &npub, &hex);
                print_json(
                    &json!({
                        "npub": npub,
                        "hex": hex,
                    }),
                    ap.output,
                    0,
                    "",
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not added to subscription filter.",
                    &ap.npub_to_hex[i]
                );
                print_json(
                    &json!({
                        "npub": ap.npub_to_hex[i],
                        "error": "invalid key",
                    }),
                    ap.output,
                    0,
                    "",
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::ConversionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --cli_hex_to_npub CLI argument
pub(crate) fn cli_hex_to_npub(ap: &Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.hex_to_npub.len();
    let mut i = 0;
    while i < num {
        match str_to_pubkeys(&ap.hex_to_npub[i]) {
            Ok((npub, hex)) => {
                debug!("Valid key. Npub {:?}, Hex: {:?}.", &npub, &hex);
                print_json(
                    &json!({
                        "npub": npub,
                        "hex": hex,
                    }),
                    ap.output,
                    0,
                    "",
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not added to subscription filter.",
                    &ap.hex_to_npub[i]
                );
                print_json(
                    &json!({
                        "hex": ap.hex_to_npub[i],
                        "error": "invalid key",
                    }),
                    ap.output,
                    0,
                    "",
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::ConversionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --cli_get_pubkey_entity CLI argument
pub(crate) async fn cli_get_pubkey_entity(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.get_pubkey_entity.len();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.get_pubkey_entity[i]) {
            Ok(pkey) => {
                debug!(
                    "Valid key. Key {:?}, Hex: {:?}.",
                    &ap.subscribe_pubkey[i],
                    pkey.to_string()
                );
                match client.get_entity_of_pubkey(pkey).await {
                    Ok(entity) => {
                        debug!(
                            "Valid key. Key {:?}, Hex: {:?}, Entity: {:?}",
                            &ap.subscribe_pubkey[i],
                            pkey.to_string(),
                            entity
                        );
                        print_json(
                            &json!({
                                "hex": pkey.to_string(),
                                "entity": format!("{:?}",entity),
                            }),
                            ap.output,
                            0,
                            "",
                        );
                    }
                    Err(ref e) => {
                        debug!(
                            "Valid key. Key {:?}, Hex: {:?}, Entity error: {:?}",
                            &ap.subscribe_pubkey[i],
                            pkey.to_string(),
                            e
                        );
                        print_json(
                            &json!({
                                "hex": pkey.to_string(),
                                "error": format!("{:?}",e),
                            }),
                            ap.output,
                            0,
                            "",
                        );
                    }
                }
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. No attempt made to determine entity.",
                    &ap.get_pubkey_entity[i]
                );
                print_json(
                    &json!({
                        "key": ap.get_pubkey_entity[i],
                        "error": "invalid key",
                    }),
                    ap.output,
                    0,
                    "",
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::GetEntityFailed)
    } else {
        Ok(())
    }
}

/// Handle the --subscribe-pubkey CLI argument, moving pkeys from CLI args into creds data structure
pub(crate) async fn cli_subscribe_pubkey(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.subscribe_pubkey.len();
    let mut pubkeys = Vec::new();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.subscribe_pubkey[i]) {
            Ok(pkey) => {
                pubkeys.push(pkey);
                debug!(
                    "Valid key added to subscription filter. Key {:?}, Hex: {:?}.",
                    &ap.subscribe_pubkey[i],
                    pkey.to_string()
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not added to subscription filter.",
                    &ap.subscribe_pubkey[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    ap.creds.subscribed_pubkeys.append(&mut pubkeys);
    ap.creds.subscribed_pubkeys.dedup_by(|a, b| a == b);
    if err_count != 0 {
        Err(Error::SubscriptionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --subscribe-author CLI argument, moving authors from CLI args into creds data structure
pub(crate) async fn cli_subscribe_author(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.subscribe_author.len();
    let mut authors = Vec::new();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.subscribe_author[i]) {
            Ok(pkey) => {
                authors.push(pkey);
                debug!(
                    "Valid key added to subscription filter. Key {:?}, {:?}.",
                    &ap.subscribe_author[i], pkey
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not added to subscription filter.",
                    &ap.subscribe_author[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    ap.creds.subscribed_authors.append(&mut authors);
    ap.creds.subscribed_authors.dedup_by(|a, b| a == b);
    if err_count != 0 {
        Err(Error::SubscriptionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --subscribe-channel CLI argument, moving pkeys from CLI args into creds data structure
pub(crate) async fn cli_subscribe_channel(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.subscribe_channel.len();
    let mut pubkeys = Vec::new();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.subscribe_channel[i]) {
            Ok(pkey) => {
                pubkeys.push(pkey);
                debug!(
                    "Valid key added to subscription filter. Key {:?}, Hex: {:?}.",
                    &ap.subscribe_channel[i],
                    pkey.to_string()
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not added to subscription filter.",
                    &ap.subscribe_channel[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    ap.creds.subscribed_channels.append(&mut pubkeys);
    ap.creds.subscribed_channels.dedup_by(|a, b| a == b);
    if err_count != 0 {
        Err(Error::SubscriptionFailed)
    } else {
        Ok(())
    }
}

/// Utility function to print JSON object as JSON or as plain text
/// depth: depth in nesting, on first call use 0.
// see https://github.com/serde-rs/json
pub(crate) fn print_json(jsonv: &Value, output: Output, depth: u32, separator: &str) {
    trace!("{:?}", jsonv);
    match output {
        Output::Text => {
            if depth != 0 {
                print!("    ");
            }
            if jsonv.is_object() {
                // if it is an object, check recursively
                for (key, val) in jsonv.as_object().unwrap() {
                    print!("{}:", key);
                    print_json(val, output, depth + 1, separator);
                    print!("    ");
                }
            } else if jsonv.is_boolean() {
                print!("{}", jsonv);
            } else if jsonv.is_null() {
                print!(""); // print nothing
            } else if jsonv.is_string() {
                print!("{}", jsonv);
            } else if jsonv.is_number() {
                print!("{}", jsonv);
            } else if jsonv.is_array() {
                print!("[ ");
                print!("{}", separator);
                let mut i = 0;
                while i < jsonv.as_array().unwrap().len() {
                    if i > 0 {
                        print!(",    ");
                    }
                    print_json(&jsonv[i], output, depth + 1, separator);
                    i += 1;
                    println!();
                }
                print!("{}", separator);
                print!(" ]");
            } else {
                debug!("not implemented type in print_json()");
                print!("{}", jsonv.to_string(),);
            }
            if depth == 0 {
                println!();
            }
        }
        Output::JsonSpec => (),
        _ => {
            // This can panic if output is piped and pipe is broken by receiving process
            println!("{}", jsonv.to_string(),);
        }
    }
}

/// Handle the --whoami CLI argument
pub(crate) fn cli_whoami(ap: &Args) -> Result<(), Error> {
    print_json(
        &json!({
            "name": ap.creds.metadata.name.clone(),
            "display_name": ap.creds.metadata.display_name.clone(),
        }),
        ap.output,
        0,
        "",
    );
    Ok(())
}

/// We need your code contributions! Please add features and make PRs! :pray: :clap:
#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut ap = Args::parse();

    eprintln!("If you know Rust a bit, if you are interested in Nostr, ");
    eprintln!("then please consider making a code contribution. ");
    eprintln!("At the very least give it a star on Github. ");
    eprintln!("Star and make PRs at: https://github.com/8go/nostr-commander-rs ");

    // handle log level and debug options
    let env_org_rust_log = env::var("RUST_LOG").unwrap_or_default().to_uppercase();
    // println!("Original log_level option is {:?}", ap.log_level);
    // println!("Original RUST_LOG is {:?}", &env_org_rust_log);
    if ap.debug > 0 {
        // -d overwrites --log-level
        ap.log_level = LogLevel::Debug
    }
    if ap.log_level.is_none() {
        ap.log_level = LogLevel::from_str(&env_org_rust_log, true).unwrap_or(LogLevel::Error);
    }
    // overwrite environment variable, important because it might have been empty/unset
    env::set_var("RUST_LOG", ap.log_level.to_string());

    // set log level e.g. via RUST_LOG=DEBUG cargo run, use newly set venv var value
    // Send *all* output from Debug to Error to stderr
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(Level::from_str(&ap.log_level.to_string()).unwrap_or(Level::ERROR))
        .init();
    debug!("Original RUST_LOG env var is {}", env_org_rust_log);
    debug!(
        "Final RUST_LOG env var is {}",
        env::var("RUST_LOG").unwrap_or_default().to_uppercase()
    );
    debug!("Final log_level option is {:?}", ap.log_level);
    if enabled!(Level::TRACE) {
        debug!("Log level is set to TRACE.");
    } else if enabled!(Level::DEBUG) {
        debug!("Log level is set to DEBUG.");
    }
    debug!("Version is {}", get_version());
    debug!("Package name is {}", get_pkg_name());
    debug!("Repo is {}", get_pkg_repository());
    debug!("Arguments are {:?}", ap);

    match ap.version {
        None => (),                     // do nothing
        Some(None) => crate::version(), // print version
        Some(Some(Version::Check)) => crate::version_check(),
    }
    if ap.usage {
        crate::usage();
        return Ok(());
    };
    if ap.help {
        crate::help();
        return Ok(());
    };
    if ap.manual {
        crate::manual();
        return Ok(());
    };
    if ap.readme {
        crate::readme().await;
        return Ok(());
    };
    if ap.contribute {
        crate::contribute();
    };

    if ap.create_user {
        match crate::cli_create_user(&mut ap) {
            Ok(()) => {
                info!("User created successfully.");
            }
            Err(ref e) => {
                error!("Creating a user failed or credentials information could not be written to disk. Check your arguments and try --create-user again. Reported error is: {:?}", e);
                return Err(Error::CreatingUserFailed);
            }
        }
    } else {
        match crate::read_credentials(&mut ap) {
            Ok(()) => {
                info!("User created successfully.");
            }
            Err(ref e) => {
                error!("Credentials file does not exists or cannot be read. Try creating a user first with --create-user. Check your arguments and try again. Worst case if file is corrupted or lost, consider doing a '--delete-user' to clean up, then perform a new '--create-user'. {:?}.", e);
                return Err(Error::ReadingCredentialsFailed);
            }
        }
    }
    // credentials are filled now

    debug!("Welcome to nostr-commander-rs");

    let my_keys = Keys::from_bech32(&ap.creds.secret_key_bech32)?;

    // Show public key
    if ap.show_public_key {
        debug!(
            "Loaded public key in Nostr format is : {:?}",
            my_keys.public_key_as_str()
        );
        debug!(
            "Loaded public key in Bech32 format is: {:?}",
            ap.creds.public_key_bech32
        );
    }
    // Show secret key
    if ap.show_secret_key {
        debug!(
            "Loaded secret key in Nostr format is : {:?}",
            my_keys.secret_key_as_str()
        );
        debug!(
            "Loaded secret key in Bech32 format is: {:?}",
            ap.creds.secret_key_bech32
        );
    }
    // whoami
    if ap.whoami {
        cli_whoami(&ap)?;
    }
    // npub_to_hex
    if !ap.npub_to_hex.is_empty() {
        match cli_npub_to_hex(&ap) {
            Ok(()) => {
                info!("Converting keys from npub to hex successful.");
            }
            Err(ref e) => {
                error!(
                    "Converting keys from npub to hex failed. Reported error is: {:?}",
                    e
                );
            }
        }
    }
    // hex_to_npub
    if !ap.hex_to_npub.is_empty() {
        match cli_hex_to_npub(&ap) {
            Ok(()) => {
                info!("Converting keys from hex to npub successful.");
            }
            Err(ref e) => {
                error!(
                    "Converting keys from hex to npub failed. Reported error is: {:?}",
                    e
                );
            }
        }
    }
    // Create new client
    let mut client = Client::new(&my_keys);

    match add_relays_from_creds(&mut client, &mut ap) {
        Ok(()) => {
            info!("Adding relays from credentials to client successful.");
        }
        Err(ref e) => {
            error!(
                "Adding relays from credentials to client failed. Reported error is: {:?}",
                e
            );
        }
    }
    // todo clean up code to separate better local action from client/remote action
    // Add relays, if --create-user the relays have already been added
    if !ap.add_relay.is_empty() && !ap.create_user {
        match crate::cli_add_relay(&mut client, &mut ap) {
            Ok(()) => {
                info!("add_relay successful.");
            }
            Err(ref e) => {
                error!("add_relay failed. Reported error is: {:?}", e);
            }
        }
    }

    if ap.listen
        || !ap.publish_pow.is_empty()
        || !ap.publish.is_empty()
        || !ap.dm.is_empty()
        || !ap.send_channel_message.is_empty()
        || !ap.subscribe_pubkey.is_empty()
        || !ap.subscribe_author.is_empty()
        || !ap.subscribe_channel.is_empty()
        || !ap.get_pubkey_entity.is_empty()
    {
        // todo avoid connect_...()  call if not relay action is needed and everything can be done locally.
        // todo avoid connect...() if no client is needed.
        //
        // also do a wait on create-user ?
        if ap.listen {
            match client.connect().await {
                Ok(()) => {
                    info!("connect successful.");
                }
                Err(ref e) => {
                    error!(
                        "connect failed. Could not connect to relays. Reported error is: {:?}",
                        e
                    );
                    return Err(Error::CannotConnectToRelays);
                }
            }
        } else {
            // Connect to relays, WAIT for connection, and keep connection alive
            match client.connect_and_wait().await {
                Ok(()) => {
                    info!("connect_and_wait successful.");
                }
                Err(ref e) => {
                    error!(
                    "connect_and_wait failed. Could not connect to relays. Reported error is: {:?}",
                    e
                );
                    return Err(Error::CannotConnectToRelays);
                }
            }
        }
    }

    if ap.create_user {
        // let metadata = Metadata::new()
        //     .name("username")
        //     .display_name("My Username")
        //     .about("Description")
        //     .picture(Url::from_str("https://example.com/avatar.png")?)
        //     .nip05("username@example.com");

        // Update profile metadata
        match client.update_profile(ap.creds.metadata.clone()).await {
            Ok(()) => {
                info!("update_profile successful.");
            }
            Err(ref e) => {
                error!(
                "update_profile failed. Could not update profile with metadata. Reported error is: {:?}",
                e
            );
            }
        }
    }

    // Set contacts, first in local file, second in client
    if ap.add_contact {
        match crate::cli_add_contact(&client, &mut ap).await {
            Ok(()) => {
                info!("add_contact successful.");
            }
            Err(ref e) => {
                error!("add_contact failed. Reported error is: {:?}", e);
            }
        }
    }
    if ap.remove_contact {
        match crate::cli_remove_contact(&client, &mut ap).await {
            Ok(()) => {
                info!("remove_contact successful.");
            }
            Err(ref e) => {
                error!("remove_contact failed. Reported error is: {:?}", e);
            }
        }
    }
    ap.creds.contacts.dedup_by(|a, b| a.alias == b.alias);
    match client.set_contact_list(ap.creds.contacts.clone()).await {
        Ok(()) => {
            info!("set_contact_list successful.");
        }
        Err(ref e) => {
            error!("set_contact_list failed. Reported error is: {:?}", e);
        }
    }
    if ap.show_contacts {
        if ap.output.is_text() {
            for c in &ap.creds.contacts {
                print_json(&json!(c), ap.output, 0, "");
            }
        } else {
            print_json(&json!({"contacts": ap.creds.contacts}), ap.output, 0, "");
        }
    }
    // ap.creds.save(get_credentials_actual_path(&ap))?; // do it later

    // Get pubkey entity
    if !ap.get_pubkey_entity.is_empty() {
        match crate::cli_get_pubkey_entity(&client, &mut ap).await {
            Ok(()) => {
                info!("get_pubkey_entity successful.");
            }
            Err(ref e) => {
                error!("get_pubkey_entity failed. Reported error is: {:?}", e);
            }
        }
    }

    // Publish a text note
    if !ap.publish.is_empty() {
        match crate::cli_publish(&client, &mut ap).await {
            Ok(()) => {
                info!("publish successful.");
            }
            Err(ref e) => {
                error!("publish failed. Reported error is: {:?}", e);
            }
        }
    }
    // Publish a POW text note
    if !ap.publish_pow.is_empty() {
        match crate::cli_publish_pow(&client, &mut ap).await {
            Ok(()) => {
                info!("publish_pow successful.");
            }
            Err(ref e) => {
                error!("publish_pow failed. Reported error is: {:?}", e);
            }
        }
    }
    // Send DMs
    if !ap.dm.is_empty() {
        match crate::cli_dm(&client, &mut ap).await {
            Ok(()) => {
                info!("dm successful.");
            }
            Err(ref e) => {
                error!("dm failed. Reported error is: {:?}", e);
            }
        }
    }
    // Send channel messages
    if !ap.send_channel_message.is_empty() {
        match crate::cli_send_channel_message(&client, &mut ap).await {
            Ok(()) => {
                info!("send-channel-message successful.");
            }
            Err(ref e) => {
                error!("send-channel-message failed. Reported error is: {:?}", e);
            }
        }
    }

    // Subscribe keys
    if !ap.subscribe_pubkey.is_empty() {
        match crate::cli_subscribe_pubkey(&mut client, &mut ap).await {
            Ok(()) => {
                debug!("subscribe_pubkey successful. Subscriptions synchronized with credentials file.");
            }
            Err(ref e) => {
                error!("subscribe_pubkey failed. Reported error is: {:?}", e);
            }
        }
    }
    if !ap.creds.subscribed_pubkeys.is_empty() && ap.listen {
        let mut ksf: SubscriptionFilter;
        ksf = SubscriptionFilter::new().pubkeys(ap.creds.subscribed_pubkeys.clone());
        if ap.limit_number != 0 {
            ksf = ksf.limit(ap.limit_number);
        }
        if ap.limit_days != 0 {
            ksf = ksf.since((Utc::now() - Duration::days(ap.limit_days)).timestamp() as u64);
        }
        if ap.limit_hours != 0 {
            ksf = ksf.since((Utc::now() - Duration::hours(ap.limit_hours)).timestamp() as u64);
        }
        if ap.limit_future_days != 0 {
            ksf = ksf.until((Utc::now() - Duration::days(ap.limit_future_days)).timestamp() as u64);
        }
        if ap.limit_future_hours != 0 {
            ksf =
                ksf.until((Utc::now() - Duration::hours(ap.limit_future_hours)).timestamp() as u64);
        }
        match client.subscribe(vec![ksf]).await {
            Ok(()) => {
                info!("subscribe to pubkeys successful.");
            }
            Err(ref e) => {
                error!("subscribe to pubkeys failed. Reported error is: {:?}", e);
            }
        }
    }
    // Subscribe authors
    if !ap.subscribe_author.is_empty() {
        match crate::cli_subscribe_author(&mut client, &mut ap).await {
            Ok(()) => {
                debug!("subscribe_author successful. Subscriptions synchronized with credentials file.");
            }
            Err(ref e) => {
                error!("subscribe_author failed. Reported error is: {:?}", e);
            }
        }
    }
    if !ap.creds.subscribed_authors.is_empty() && ap.listen {
        let mut asf: SubscriptionFilter;
        asf = SubscriptionFilter::new().authors(ap.creds.subscribed_authors.clone());
        if ap.limit_number != 0 {
            asf = asf.limit(ap.limit_number);
        }
        if ap.limit_days != 0 {
            asf = asf.since((Utc::now() - Duration::days(ap.limit_days)).timestamp() as u64);
        }
        if ap.limit_hours != 0 {
            asf = asf.since((Utc::now() - Duration::hours(ap.limit_hours)).timestamp() as u64);
        }
        if ap.limit_future_days != 0 {
            asf = asf.until((Utc::now() - Duration::days(ap.limit_future_days)).timestamp() as u64);
        }
        if ap.limit_future_hours != 0 {
            asf =
                asf.until((Utc::now() - Duration::hours(ap.limit_future_hours)).timestamp() as u64);
        }
        match client.subscribe(vec![asf]).await {
            Ok(()) => {
                info!("subscribe to authors successful.");
            }
            Err(ref e) => {
                error!("subscribe to authors failed. Reported error is: {:?}", e);
            }
        }
    }
    // Subscribe channels
    if !ap.subscribe_channel.is_empty() {
        match crate::cli_subscribe_channel(&mut client, &mut ap).await {
            Ok(()) => {
                debug!("subscribe_channel successful. Subscriptions synchronized with credentials file.");
            }
            Err(ref e) => {
                error!("subscribe_channel failed. Reported error is: {:?}", e);
            }
        }
    }
    if !ap.creds.subscribed_channels.is_empty() && ap.listen {
        let mut csf: SubscriptionFilter;
        csf = SubscriptionFilter::new().events(ap.creds.subscribed_channels.clone());
        if ap.limit_number != 0 {
            csf = csf.limit(ap.limit_number);
        }
        if ap.limit_days != 0 {
            csf = csf.since((Utc::now() - Duration::days(ap.limit_days)).timestamp() as u64);
        }
        if ap.limit_hours != 0 {
            csf = csf.since((Utc::now() - Duration::hours(ap.limit_hours)).timestamp() as u64);
        }
        if ap.limit_future_days != 0 {
            csf = csf.until((Utc::now() - Duration::days(ap.limit_future_days)).timestamp() as u64);
        }
        if ap.limit_future_hours != 0 {
            csf =
                csf.until((Utc::now() - Duration::hours(ap.limit_future_hours)).timestamp() as u64);
        }
        match client.subscribe(vec![csf]).await {
            Ok(()) => {
                info!("subscribe to channels successful.");
            }
            Err(ref e) => {
                error!("subscribe to channels failed. Reported error is: {:?}", e);
            }
        }
    }
    ap.creds.save(get_credentials_actual_path(&ap))?;

    // notices will be published even if we do not go into handle_notification event loop
    // Design choice: Do not automatically listen when subscriptions exist, only listen to subscriptions if --listen is set.
    if ap.listen
    // || !ap.creds.subscribed_authors.is_empty()
    // || !ap.creds.subscribed_pubkeys.is_empty()
    {
        let num =
            ap.publish.len() + ap.publish_pow.len() + ap.dm.len() + ap.send_channel_message.len();
        if num == 1 {
            info!(
                "You should be receiving {:?} 'OK' message with event id for the notice once it has been relayed.",
                num
            );
        } else if num > 1 {
            info!(
                "You should be receiving {:?} 'OK' messages with event ids, one for each notice that has been relayed.",
                num
            );
        }
        // Handle notifications
        match client
            .handle_notifications(|notification| {
                debug!("Notification: {:?}", notification);
                match notification {
                    ReceivedEvent(ev) => {
                        debug!("Event-Event: content {:?}, kind {:?}", ev.content, ev.kind);
                    }
                    ReceivedMessage(msg) => {
                        // debug!("Message: {:?}", msg);
                        match msg {
                            RelayMessage::Ok {event_id, status, message } => {
                                // Notification: ReceivedMessage(Ok { event_id: 123, status: true, message: "" })
                                // confirmation of notice having been relayed
                                info!("Message-OK: Notice, DM or message was relayed. Event id is {:?}. Status is {:?} and message is {:?}. You can investigate this event by looking it up on https://nostr.com/e/{}", event_id, status, message, event_id.to_string());
                                print_json(
                                    &json!({"event_type": "RelayMessage::Ok",
                                        "event_type_meaning": "Notice, DM or message was relayed successfully.",
                                        "event_id": event_id,
                                        "status": status,
                                        "message": message,
                                        "event_url": "https://nostr.com/e/".to_string() + &event_id.to_string(),
                                        "event_url_meaning": "You can investigate this event by looking up the event URL.",
                                    }) ,
                                    ap.output,0,""
                                );
                            },
                            RelayMessage::Notice { message } => {
                                debug!("Message-Notice: {:?}", message);
                            }
                            RelayMessage::Event {event, subscription_id}=> {
                                // kind: Base(ChannelMessage) and Base(TextNote) and Base(Reaction)
                                let mut tags = "".to_owned();
                                let mut first = true;
                                for t in &event.tags {
                                    match t.kind() {
                                        Ok(TagKind::P) => {
                                            match t.content() {
                                                Some(c) => {
                                                    trace!("tag: {:?}", get_contact_alias_or_keystr_by_keystr(&ap, c));
                                                    match get_contact_alias_by_keystr(&ap, c) {
                                                        Some(a) => {
                                                            if !first { tags += ", "; };
                                                            tags += &a;
                                                            first = false;
                                                            },
                                                        _ => ()
                                                    }
                                                }
                                                None => ()
                                            }
                                        },
                                        Ok(TagKind::E) => (),
                                        Ok(TagKind::Nonce) => (),
                                        Ok(TagKind::Delegation) => todo!(),
                                        Err(_) => ()
                                    }
                                }
                                trace!("Message-Event: content {:?}, kind {:?}, from pubkey {:?}, with tags {:?}", event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey), event.tags);
                                let mut key_author = "key";
                                if is_subscribed_author(&ap, &event.pubkey) {
                                            key_author = "author";
                                            tags = get_contact_alias_or_keystr_by_key(&ap, event.pubkey);
                                        };
                                match event.kind {
                                    Kind::Base(KindBase::ContactList) => {
                                        debug!("Received Message-Event ContactList");
                                    },
                                    Kind::Base(KindBase::Reaction) => {
                                        debug!("Received Message-Event Reaction: content {:?}", event.content);
                                    },
                                    Kind::Base(KindBase::TextNote) => {
                                        info!("Subscription by {} ({}): content {:?}, kind {:?}, from pubkey {:?}", key_author, tags, event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey));
                                        print_json(
                                            &json!({
                                                "event_type": "RelayMessage::Event",
                                                "event_type_meaning": "Message was received because of subscription.",
                                                "subscribed_by": key_author,
                                                "author": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "content": event.content,
                                                // "kind": event.kind, // writes integer like '1'
                                                "kind": format!("{:?}",event.kind), // writes text like "Base(TextNote)"
                                                "from_alias": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "from_pubkey": event.pubkey,
                                                "tags": tags
                                            }) ,
                                            ap.output,0,""
                                        );
                                    },
                                    Kind::Base(KindBase::ChannelMessage) => {
                                        info!("Subscription by {} ({}): content {:?}, kind {:?}, from pubkey {:?}", key_author, tags, event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey));
                                        print_json(
                                            &json!({
                                                "event_type": "RelayMessage::Event",
                                                "event_type_meaning": "Message was received because of subscription.",
                                                "subscribed_by": key_author,
                                                "author": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "content": event.content,
                                                "kind": format!("{:?}",event.kind),
                                                "from_alias": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "from_pubkey": event.pubkey,
                                                "tags": tags
                                            }) ,
                                            ap.output,0,""
                                        );
                                    },
                                    _ => ()
                                }
                            },
                            RelayMessage::Empty => {
                                        debug!("Received Message-Event Empty");
                                    },
                            RelayMessage::EndOfStoredEvents {subscription_id} =>  {
                                        debug!("Received Message-Event EndOfStoredEvents");
                                    },
                        }
                    }
                }
                Ok(())
            })
            .await
        {
            Ok(()) => {
                info!("handle_notifications successful.");
            }
            Err(ref e) => {
                error!("handle_notifications failed. Reported error is: {:?}", e);
            }
        }
    }

    debug!("Good bye");
    Ok(())
}

/// Future test cases will be put here
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), ());
    }

    #[test]
    fn test_contribute() {
        assert_eq!(contribute(), ());
    }
}
