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
//! - run `nostr-commander-rs --manual`
//! - run `nostr-commander-rs --readme`
//!
//! For more information, see read the README.md
//! <https://github.com/8go/nostr-commander-rs/blob/main/README.md>
//! file.

#![allow(dead_code)] // crate-level allow  // Todo
#![allow(unused_variables)] // Todo
#![allow(unused_imports)] // Todo

use atty::Stream;
use chrono::Utc;
use clap::{ColorChoice, CommandFactory, Parser, ValueEnum};
use core::cmp::Ordering;
use directories::ProjectDirs;
use regex::Regex;
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
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, enabled, error, info, trace, warn, Level};
use tracing_subscriber::EnvFilter;
use update_informer::{registry, Check};

use nostr_sdk::prelude::*;
use nostr_sdk::RelayPoolNotification::{Event, Message, RelayStatus};

// /// import nostr-sdk Client related code of general kind: create_user, delete_user, etc // todo
// mod client; // todo
// use crate::client::dummy; // todo

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
/// fallback if static compile time value is None
const BIN_NAME_UNDERSCORE: &str = "nostr_commander_rs";
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

    #[error("Unsubscribe Failed")]
    UnsubscribeFailed,

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

    #[error("Unsupported CLI parameter: {0}")]
    UnsupportedCliParameter(&'static str),

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
    NostrNip04(#[from] nostr_sdk::nostr::nips::nip04::Error),

    #[error(transparent)]
    NostrKey(#[from] nostr_sdk::nostr::key::Error),

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
/// etc.
/// Alternatively, you can skip creating a user and do a fire-and-forget
/// publish like this: nostr-commander-rs --nsec nsec1SomeStrangeString --add-relay
/// "wss://some.relay.net/" --publish "test".
///   ───
/// Have a look at the repo "https://github.com/8go/nostr-commander-rs/"
/// and see if you can contribute code to improve this tool.
/// Safe!
#[derive(Clone, Debug, Parser)]
#[command(author, version,
    next_line_help = true,
    bin_name = get_prog_without_ext(),
    color = ColorChoice::Always,
    term_width = 79,
    after_help = "PS: Also have a look at scripts/nostr-commander-tui.",
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
    /// Details::
    /// If used without an argument such as '--version' it will
    /// print the version number. If 'check' is added ('--version check')
    /// then the program connects to https://crates.io and gets the version
    /// number of latest stable release. There is no "calling home"
    /// on every run, only a "check crates.io" upon request. Your
    /// privacy is protected. New release is neither downloaded,
    /// nor installed. It just informs you.
    #[arg(short, long, value_name = "CHECK")]
    version: Option<Option<Version>>,

    /// Prints a very short help summary.
    /// Details:: See also --help, --manual and --readme.
    #[arg(long)]
    usage: bool,

    /// Prints short help displaying about one line per argument.
    /// Details:: See also --usage, --manual and --readme.
    #[arg(short, long)]
    help: bool,

    /// Prints long help.
    /// Details:: This is like a man page.
    /// See also --usage, --help and --readme.
    #[arg(long)]
    manual: bool,

    /// Prints README.md file, the documenation in Markdown.
    /// Details:: The README.md file will be downloaded from
    /// Github. It is a Markdown file and it is best viewed with a
    /// Markdown viewer.
    /// See also --usage, --help and --manual.
    #[arg(long)]
    readme: bool,

    /// Overwrite the default log level.
    /// Details::
    /// If not used, then the default
    /// log level set with environment variable 'RUST_LOG' will be used.
    /// If used, log level will be set to 'DEBUG' and debugging information
    /// will be printed.
    /// '-d' is a shortcut for '--log-level DEBUG'.
    /// If used once as in '-d' it will set and/or overwrite
    /// --log-level to '--log-level debug'.
    /// If used twice as in '-d -d' it will set and/or overwrite
    /// --log-level to '--log-level debug debug'.
    /// And third or futher occurance of '-d' will be ignored.
    /// See also '--log-level'. '-d' takes precedence over '--log-level'.
    /// Additionally, have a look also at the option '--verbose'.
    #[arg(short, long,  action = clap::ArgAction::Count, default_value_t = 0u8, )]
    debug: u8,

    /// Set the log level by overwriting the default log level.
    /// Details::
    /// If not used, then the default
    /// log level set with environment variable 'RUST_LOG' will be used.
    /// If used with one value specified this value is assigned to the
    /// log level of matrix-commander-rs.
    /// If used with two values specified the first value is assigned to the
    /// log level of matrix-commander-rs. The second value is asigned to the
    /// lower level modules.
    /// More than two values should not be specified.
    /// --debug overwrites -log-level.
    /// See also '--debug' and '--verbose'.
    /// Alternatively you can use the RUST_LOG environment variable.
    /// An example use of RUST_LOG is to use neither --log-level nor --debug,
    /// and to set RUST_LOG="error,matrix_commander_rs=debug" which turns
    /// off debugging on all lower level modules and turns debugging on only
    /// for matrix-commander-rs.
    // Possible values are
    // '{trace}', '{debug}', '{info}', '{warn}', and '{error}'.
    #[arg(long, value_delimiter = ' ', num_args = 1..3, ignore_case = true, )]
    log_level: Option<Vec<LogLevel>>,

    /// Set the verbosity level.
    /// Details::
    /// If not used, then verbosity will be
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
    //
    //
    /// Specify a path to a file containing credentials.
    /// Details::
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

    /// Create a new user, i.e. a new key pair.
    /// Details::
    /// This is usually
    /// done only once at the beginning. If you ever want to wipe
    /// this user, use '--delete-user' which deletes the key
    /// pair. Use this option in combination with --name,
    ///  --display_name, --about, --picture, --nip05, and
    /// --nsec.
    /// Also highly recommended that you use this option
    /// together with --add-relay.
    /// Add --nsec as option to import your existing nsec
    /// private key, otherwise a new private key will be
    /// generated for you.
    #[arg(long, alias = "create-key", default_value_t = false)]
    create_user: bool,

    /// Delete the current user, i.e. delete the current key pair.
    /// Details::
    /// This will erase the key pair and other associated information
    /// like user name, display name, etc. Afterwards one can create
    /// a new user with '--create-user'.
    #[arg(long, alias = "delete-key", default_value_t = false)]
    delete_user: bool,

    /// Specify an optional user name.
    /// Details::
    /// Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "USER_NAME")]
    name: Option<String>,

    /// Specify an optional display name.
    /// Details::
    /// Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "DISPLAY_NAME")]
    display_name: Option<String>,

    /// Specify an optional description.
    /// Details::
    /// Used together with
    /// '--create-user'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "DESCRIPTION")]
    about: Option<String>,

    /// Specify an optional picture or avatar.
    /// Details:: Used together with
    /// '--create-user'. Provide a URL like 'https://example.com/avatar.png'.
    // or a local file like 'file://somepath/someimage.jpg'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide this URL 'none:'.
    #[arg(long, value_name = "URL")]
    picture: Option<Url>,

    /// Specify an optional nip05 name.
    /// Details::
    /// Used together with
    /// '--create-user'. Provide a nip05 name like 'john@example.org'.
    /// If this option is not set during '--create-user', the information
    /// will be queried via the keyboard. If you want to set it to empty
    /// and not be queried, provide an empty string ''.
    #[arg(long, value_name = "NIP05_ID")]
    nip05: Option<String>,

    /// Provide one private key.
    /// Details:: It has the form 'nsec1SomeStrangeString'.
    /// Alternatively you can use the Hex form of the private key.
    /// Since this is your private key, you must protect it. Don't put
    /// this key into a script, into Github, etc.
    /// If --nsec is provided it will be used instead of the private key in
    /// the credentials file.
    /// This argument is used e.g. in combination with argument
    /// --publish.
    /// If you are using --nsec without a credentials file most likely
    /// you want to also use -ad-relay argument.
    /// If --nsec is used without --create-user then the credentials
    /// file will not be modified or will not be created.
    #[arg(long, value_name = "PRIVATE_KEY")]
    nsec: Option<String>,

    /// Publish one or multiple notes.
    /// Details::
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
    /// Details::
    /// Use also '--pow-difficulty' to specify difficulty.
    /// See also '--publish' to see how shortcut characters
    /// '-' (pipe) and '_' (streamed pipe) are handled.
    /// Disabled since version nostr-commander-rs 0.2.0 (nostr-sdk 0.21).
    #[arg(long, alias = "pow", value_name = "NOTE", num_args(0..), )]
    publish_pow: Vec<String>, // ToDo: remove this option

    /// Send one or multiple DMs to one given user.
    /// Details::
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
    /// Details::
    /// The single destination channel is specified via its hash.
    /// See here for a channel list: https://damus.io/channels/.
    /// The first argument
    /// is the channel hash, all further arguments are texts to be
    /// sent. E.g.
    /// '-send_channel_message "SomeChannelHash" "First msg" "Second msg"'.
    // or '--send_channel_message joe "How about pizza tonight?"'.
    /// See also '--publish' to see how shortcut characters
    /// '-' (pipe) and '_' (streamed pipe) are handled.
    /// Optionally you can provide a relay to be used for the channel send
    /// by using --relay. See --relay. If --relay has values the first value
    /// from --relay will be used as relay. If --relay is not used, then
    /// the first relay in the relay list in the credentials configuration
    /// file will be used.
    #[arg(long, alias = "chan", value_name = "HASH+MSGS", num_args(0..), )]
    send_channel_message: Vec<String>,

    /// Add one or multiple relays.
    /// Details::
    /// A relay is specified via a URI
    /// that looks like 'wss://some.relay.org'. You can find relays
    /// by looking at https://github.com/aljazceru/awesome-nostr#instances.
    /// Sampler relay registries are: https://nostr-registry.netlify.app/,
    /// https://nostr.info/, or https://nostr.watch/.
    /// Examples: "wss://relay.damus.io", "wss://nostr.openchain.fr".
    /// See also '--proxy'.
    #[arg(long, value_name = "RELAY_URI", num_args(0..), )]
    add_relay: Vec<Url>,

    /// Specify a proxy for relays.
    /// Details:: Used by --add-relay.
    /// Note that this proxy will be applied to all of the relays specified
    /// with --add-relay. If you have 3 relays with 3 different proxies, then
    /// run the --add-relay command 3 times with 1 relay and 1 proxy each time.
    /// An example proxy for the Tor network looks something like "127.0.0.1:9050".
    /// If you want to use Tor via a proxy, to assure that no information
    /// leaks you must use only one relay, i.e. the Tor relay.
    /// If more then one relays are configured, data will be communicated to
    /// and from all relays.
    /// A possible relay that you can use together with a Tor proxy is
    /// "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion".
    #[arg(long)]
    proxy: Option<SocketAddr>,

    /// Remove one or multiple relays from local config file.
    /// Details:: See --add-relay.
    #[arg(long, value_name = "RELAY_URI", num_args(0..), )]
    remove_relay: Vec<Url>,

    // todo tag
    /// Specify one or multiple tags to attach to notes or DMs.
    /// Details:: Not yet implemented.
    #[arg(long)]
    tag: Vec<String>,

    /// Display current metadata.
    /// Details:: shows data in your config file.
    #[arg(long, default_value_t = false)]
    show_metadata: bool,

    /// Modify existing metadata of the user.
    /// Details::
    /// Use this option in combination with --name,
    ///  --display_name, --about, --picture, and --nip05.
    #[arg(long, default_value_t = false)]
    change_metadata: bool,

    /// Specify optional proof-of-work (POW) difficulty.
    /// Details::
    /// Use with '--publish_pow' to specify difficulty.
    /// If not specified the default will be used.
    #[arg(long, value_name = "DIFFICULTY", default_value_t = POW_DIFFICULTY_DEFAULT, )]
    pow_difficulty: u8,

    /// Show public key.
    /// Details:: Displays your own public key. You can share this
    /// with your friends or the general public.
    #[arg(long, default_value_t = false)]
    show_public_key: bool,

    /// Show private, secret key.
    /// Details:: Protect this key. Do not share this with anyone.
    #[arg(long, default_value_t = false)]
    show_secret_key: bool,

    /// Print the user name used by "nostr-commander-rs".
    /// Details::
    /// One can get this information also by looking at the
    /// credentials file or by using --show-metadata.
    #[arg(long)]
    whoami: bool,

    /// Select an output format.
    /// Details:: This option decides on how the output is presented.
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
    /// Details::
    /// This option listens to events and messages forever. To stop, type
    /// Control-C on your keyboard. You want to listen if you want
    /// to get the event ids for published notices.
    /// Subscriptions do not automatically turn listening on.
    /// If you want to listen to your subscriptions, you must use
    /// --listen.
    #[arg(short, long, default_value_t = false)]
    listen: bool,

    /// Add one or more contacts.
    /// Details:: Must be used in combination with
    /// --alias, --key, --relay. If you want to add N new contacts,
    /// use --add-contact and provide exactly N entries in each
    /// of the 3 extra arguments. E.g. --add-contact --alias jane joe
    /// --key npub1JanesPublicKey npub1JoesPublicKey
    /// --relay "wss://janes.relay.org" "wss://joes.relay.org".
    /// Aliases must be unique. Alias can be seen as a nickname.
    #[arg(long, default_value_t = false)]
    add_contact: bool,

    /// Remove one or more contacts.
    /// Details:: Must be used in combination with
    /// --alias. For each entry in --alias the corresponding contact will
    /// be removed. E.g. --remove-contact --alias jane joe.
    #[arg(long, default_value_t = false)]
    remove_contact: bool,

    /// Display current contacts.
    /// Details:: Prints your contact list.
    #[arg(long, default_value_t = false)]
    show_contacts: bool,

    /// Provide one or multiple aliases (nicknames).
    /// Details:: This is used in combination with arguments
    /// --add-contact and --remove-contact.
    #[arg(long, value_name = "ALIAS", num_args(0..), )]
    alias: Vec<String>,

    /// Provide one or multiple public keys.
    /// Details:: This is used in combination with argument
    /// --add-contact. They have the form 'npub1SomeStrangeString'.
    /// Alternatively you can use the Hex form of the public key.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    key: Vec<String>,

    /// Provide one or multiple relays.
    /// Details:: This is used in combination with arguments
    /// --add-contact and --send_channel_message.
    /// Relays have the form 'wss://some.relay.org'.
    #[arg(long, value_name = "RELAY", num_args(0..), )]
    relay: Vec<Url>,

    /// Convert one or multiple public keys from Npub to Hex.
    /// Details:: Converts public keys in Bech32 format ('npub1...') into
    /// the corresponding 'hex' format.
    /// See also --hex-to-npub.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    npub_to_hex: Vec<String>,

    /// Convert one or multiple public keys from Hex to Npub.
    /// Details:: Converts public keys in 'hex' format into
    /// the corresponding Bech32 ('npub1...') format.
    /// See also --npub-to-hex.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    hex_to_npub: Vec<String>,

    /// Subscribe to one or more public keys.
    /// Details:: Specify each
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

    /// Subscribe to public channels with one or more hashes of channels.
    /// Details:: Specify each
    /// hash in form of 'npub1SomePublicKey'.
    /// Alternatively you can use the Hex form of the public key.
    /// Sometimes the hash of a public channel is referred to as
    /// channel id, sometimes also as public channel key.
    /// See here for a channel list: https://damus.io/channels/.
    /// Provide hashes that represent public channels (see --get-pubkey-entity).
    /// See also --subscribe-pubkey and --subscribe-author which are different.
    #[arg(long, value_name = "HASH", num_args(0..), )]
    subscribe_channel: Vec<String>,

    /// Unsubscribe from public key.
    /// Details:: Removes one or multiple public keys from the
    /// public key subscription list.
    /// See --subscribe-pubkey.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    unsubscribe_pubkey: Vec<String>,

    /// Unsubscribe from author.
    /// Details:: Removes one or multiple public keys from the
    /// author subscription list.
    /// See --subscribe-author.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    unsubscribe_author: Vec<String>,

    /// Unsubscribe from public channel.
    /// Details:: Removes one or multiple public keys from the
    /// public channel subscription list.
    /// See --subscribe-channel.
    #[arg(long, value_name = "KEY", num_args(0..), )]
    unsubscribe_channel: Vec<String>,

    /// Limit the number of past messages to receive when subscribing.
    /// Details:: By default there is no limit (0), i.e. all old messages
    /// available to the relay will be received.
    #[arg(long, value_name = "NUMBER", default_value_t = 0)]
    limit_number: usize,

    /// Limit the messages received to the last N days when subscribing.
    /// Details:: By default there is no limit (0), i.e. all old messages
    /// available to the relay will be received.
    #[arg(long, alias = "since-days", value_name = "DAYS", default_value_t = 0)]
    limit_days: u64,

    /// Limit the messages received to the last N hours when subscribing.
    /// Details:: By default there is no limit (0), i.e. all old messages
    /// available to the relay will be received.
    #[arg(long, alias = "since-hours", value_name = "HOURS", default_value_t = 0)]
    limit_hours: u64,

    /// Limit the messages received to the next N days when subscribing.
    /// Details:: Stop receiving N days in the future.
    /// By default there is no limit (0), i.e. you will receive events forever.
    #[arg(long, alias = "until-days", value_name = "DAYS", default_value_t = 0)]
    limit_future_days: u64,

    /// Limit the messages received to the last N hours when subscribing.
    /// Details:: Stop receiving N hours in the future.
    /// By default there is no limit (0), i.e. you will receive events forever.
    #[arg(long, alias = "until-hours", value_name = "HOURS", default_value_t = 0)]
    limit_future_hours: u64,
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
            log_level: None,
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
            nsec: None,
            publish: Vec::new(),
            publish_pow: Vec::new(),
            dm: Vec::new(),
            send_channel_message: Vec::new(),
            add_relay: Vec::new(),
            remove_relay: Vec::new(),
            tag: Vec::new(),
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
            subscribe_pubkey: Vec::new(),
            subscribe_author: Vec::new(),
            subscribe_channel: Vec::new(),
            unsubscribe_pubkey: Vec::new(),
            unsubscribe_author: Vec::new(),
            unsubscribe_channel: Vec::new(),
            limit_number: 0,
            limit_days: 0,
            limit_hours: 0,
            limit_future_days: 0,
            limit_future_hours: 0,
        }
    }
}

/// A struct for the relays. These will be serialized into JSON
/// and written to the credentials.json file for permanent storage and
/// future access.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relay {
    url: Url,
    proxy: Option<SocketAddr>,
}

impl AsRef<Relay> for Relay {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Default for Relay {
    fn default() -> Self {
        Self::new(Url::parse("wss://relay.nostr.info/").unwrap(), None)
    }
}

/// implementation of Relay struct
impl Relay {
    /// Default constructor
    fn new(url: Url, proxy: Option<SocketAddr>) -> Self {
        Self { url, proxy }
    }
}

/// A struct for the credentials. These will be serialized into JSON
/// and written to the credentials.json file for permanent storage and
/// future access.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    secret_key_bech32: String, // nsec1...// private_key
    public_key_bech32: String, // npub1...
    relays: Vec<Relay>,
    metadata: Metadata,
    contacts: Vec<Contact>,
    subscribed_pubkeys: Vec<PublicKey>,
    subscribed_authors: Vec<PublicKey>,
    // todo: zzz subscribed_channels should be EventId's ?
    subscribed_channels: Vec<PublicKey>,
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
    println!("Options:");
    // let cmd = Args::command();
    // // println!("{:?}", cmd);
    // // for arg in cmd.get_arguments() {
    // //         println!("{:?}",arg);
    // // }
    // // for arg in cmd.get_arguments() {
    // //         println!("{}",arg); // bug in clap, panics
    // // }
    // for arg in cmd.get_arguments() {
    //     let s = arg.get_help().unwrap().to_string();
    //     let v: Vec<&str> = s.split("Details::").collect();
    //     let val_names = arg.get_value_names().unwrap_or(&[]);
    //     let mut pvalnames = false;
    //     match arg.get_num_args() {
    //         None => {}
    //         Some(range) => {
    //             println!("range {:?}", range);
    //             if range != clap::builder::ValueRange::EMPTY {
    //                 pvalnames = true;
    //             }
    //         }
    //     }
    //     if pvalnames {
    //         println!(
    //             "--{} [<{}>]:  {}",
    //             arg.get_long().unwrap(),
    //             val_names[0],
    //             v[0]
    //         );
    //     } else {
    //         println!("--{}: {}", arg.get_long().unwrap(), v[0]);
    //     }
    // }
    let help_str = Args::command().render_help().to_string();
    let v: Vec<&str> = help_str.split('\n').collect();
    for l in v {
        if l.starts_with("  -") || l.starts_with("      --") {
            println!("{}", &l);
        }
    }
}

/// Prints the short help
pub fn help() {
    let help_str = Args::command().render_help().to_string();
    // println!("{}", &help_str);
    // regex to remove shortest pieces "Details:: ... \n  -"
    // regex to remove shortest pieces "Details:: ... \n      --"
    // regex to remove shortest pieces "Details:: ... \nPS:"
    // 2 regex groups: delete and keep.
    // [\S\s]*? ... match anything in a non-greedy fashion
    // stop when either "PS:", "  -" or "      --" is reached
    let re = Regex::new(r"(?P<del>[ ]+Details::[\S\s]*?)(?P<keep>\nPS:|\n  -|\n      --)").unwrap();
    let after = re.replace_all(&help_str, "$keep");
    print!("{}", &after.replace("\n\n", "\n")); // remove empty lines
    println!("{}", "Use --manual to get more detailed help information.");
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
pub fn version(ap: &Args) {
    if ap.output.is_text() {
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
    } else {
        print_json(
            &json!({
                "program": get_prog_without_ext(),
                "repo": get_pkg_repository(),
                "version": get_version(),
                "icon": "NC :crab:",
            }),
            ap.output,
            0,
            "",
        );
    }
}

/// Prints the installed version and the latest crates.io-available version
pub fn version_check(ap: &Args) {
    let key1 = "Installed version";
    let value1 = get_version();
    let key2: &str;
    let value2: String;
    let key3: &str;
    let value3: String;
    let name = env!("CARGO_PKG_NAME");
    let location: String = "https://crates.io/crates/".to_owned() + name;
    let version = env!("CARGO_PKG_VERSION");
    let informer = update_informer::new(registry::Crates, name, version).check_version();
    match informer {
        Ok(Some(version)) => {
            key2 = "New available version";
            value2 = format!("{:?}", version);
            key3 = "New version available at";
            value3 = location;
        }
        Ok(None) => {
            key2 = "Status";
            value2 = "You are up-to-date. You already have the latest version.".to_owned();
            key3 = "Update required";
            value3 = "No".to_owned();
        }
        Err(ref e) => {
            key2 = "Error";
            value2 = "Could not get latest version.".to_owned();
            key3 = "Error message";
            value3 = format!("{:?}", e);
        }
    };
    print_json(
        &json!({
            key1: value1,
            key2: value2,
            key3: value3,
        }),
        ap.output,
        0,
        "",
    );
}

/// Asks the public for help
pub fn contribute(ap: &Args) {
    let text = format!(
        "{}{}{}{}{}{}",
        "This project is currently an experiment. ",
        "If you know Rust and are interested in Nostr, please have a look at the repo ",
        get_pkg_repository(),
        ". Please contribute code to improve the ",
        get_prog_without_ext(),
        " Nostr CLI client. Safe!"
    );
    print_json(
        &json!({
            "Contribute": text,
        }),
        ap.output,
        0,
        "",
    );
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
                Ok(url) => {
                    ap.creds.metadata.picture = Some(url.as_str().to_string());
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

/// Reads metadata item from keyboard.
fn get_proxy() -> Option<SocketAddr> {
    loop {
        print!("Enter proxy for relay (e.g. https://127.0.0.1:9050) or leave empty for no proxy: ");
        std::io::stdout()
            .flush()
            .expect("error: could not flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");

        match input.trim() {
            "" => {
                info!("Proxy left empty. That is okay!");
                return None;
            }
            _ => match SocketAddr::from_str(input.trim()) {
                Ok(u) => {
                    info!("proxy {:?} is accepted.", &u);
                    return Some(u);
                }
                Err(ref e) => {
                    error!(
                        "{:?} is not a valid proxy. Try again or leave empty. Reported error is {:?}.",
                        input.trim(), e
                    );
                }
            },
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
                info!("Relay left empty. That is okay!");
                repeat = false;
            }
            _ => match Url::parse(input.trim()) {
                Ok(u) => {
                    if u.scheme() == "wss" || u.scheme() == "ws" {
                        let proxy = crate::get_proxy();
                        ap.creds.relays.push(Relay::new(u.clone(), proxy));
                        info!("relay {:?} {:?} added.", &u, proxy);
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
            if ap.nsec.is_some() {
                debug!("no credentials file found or read")
            } else {
                error!(
                    "Error: failed to read credentials file {:?}. Correct path? Error reported: {:?}.",
                    get_credentials_actual_path(&ap),
                    e,
                );
            };
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
    if relay.scheme() != "wss" && relay.scheme() != "ws" {
        return false;
    } else if relay.host_str().is_none() {
        return false;
    }
    true
}

/// Handle the --create_user CLI argument
pub(crate) fn cli_create_user(ap: &mut Args) -> Result<(), Error> {
    if !ap.create_user {
        return Err(Error::MissingCliParameter);
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
    match ap.picture.clone() {
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
            if is_relay_url(&ap.add_relay[i]) {
                ap.creds
                    .relays
                    .push(Relay::new(ap.add_relay[i].clone(), ap.proxy));
            } else {
                error!(
                    "Invalid relay syntax for relay {:?}. Skipping it.",
                    ap.add_relay[i]
                )
            }
            i += 1;
        }
    }
    ap.creds.relays.dedup_by(|a, b| a.url == b.url);

    let my_keys: Keys = if ap.nsec.is_some() {
        // nsec key provided as argument, import it
        // parses from bech32 as well as from hex
        debug!("Importing private key from --nsec argument");
        Keys::new(SecretKey::parse(ap.nsec.clone().unwrap())?)
    } else {
        // Generate new keys
        debug!("A new private key is being generated for you");
        Keys::generate()
    };
    debug!("Generated private key is: {:?}", my_keys.secret_key());
    debug!("Generated public  key is: {:?}", my_keys.public_key());
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
    match my_keys.secret_key().to_bech32() {
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
pub(crate) async fn add_relays_from_creds(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0u32;
    for relay in &ap.creds.relays {
        match add_relay(client, &relay.url, relay.proxy).await {
            false => err_count += 1,
            _ => (),
        }
    }
    match err_count {
        0 => Ok(()),
        _ => Err(Error::AddRelayFailed),
    }
}

async fn add_relay(client: &mut Client, url: &Url, proxy: Option<SocketAddr>) -> bool {
    let mut result = Ok(true);
    match proxy {
        None => {
            result = client.add_relay(url).await;
        }
        Some(addr) => {
            let relaypool = client.pool();
            let mode = ConnectionMode::Proxy(addr);
            let opts = RelayOptions::new()
                .connection_mode(mode)
                .write(false)
                .retry_sec(11);
            let result = relaypool.add_relay(url, opts).await;
        }
    }
    match result {
        Ok(value) => {
            let status = if value { "successful" } else { "already added" };
            debug!(
                "add_relay...() with relay {:?} with proxy {:?}: {}.",
                url, proxy, status
            );
            true
        }
        Err(ref e) => {
            error!(
                "Error: add_relay...() returned error. Relay {:?} not added. \
                    Reported error {:?}.",
                url, e
            );
            false
        }
    }
}

/// Handle the --add_relay CLI argument.
/// Add relays from --add-relay.
pub(crate) async fn cli_add_relay(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0u32;
    let urls: Vec<&Url> = ap
        .add_relay
        .iter()
        .filter_map(|url| match is_relay_url(url) {
            true => Some(url),
            false => {
                err_count += 1;
                error!(
                    "Error: Relay {:?} is syntactically not correct. Relay not added.",
                    url,
                );
                None
            }
        })
        .collect();

    for url in urls {
        match add_relay(client, url, ap.proxy).await {
            true => ap.creds.relays.push(Relay::new(url.clone(), ap.proxy)),
            false => err_count += 1,
        }
    }

    if ap.nsec.is_none() || ap.create_user {
        debug!("Creating or updating credentials file to add new relays.");
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
    } else {
        debug!("Not creating or not updating credentials file with new relays.")
    }

    match err_count {
        0 => Ok(()),
        _ => Err(Error::AddRelayFailed),
    }
}

/// Handle the --remove-relay CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_remove_relay(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let num = ap.remove_relay.len();
    let mut i = 0;
    while i < num {
        ap.creds.relays.retain(|x| x.url != ap.remove_relay[i]);
        i += 1;
    }
    Ok(())
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
                            match client.publish_text_note(&line, []).await {
                                Ok(ref event_id) => debug!(
                                    "Publish_text_note number {:?} from pipe stream sent successfully. {:?}. event_id {:?}",
                                    i, &line, event_id
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

        match client.publish_text_note(&fnote, []).await {
            Ok(ref event_id) => debug!(
                "Publish_text_note number {:?} sent successfully. {:?}, event_id {:?}",
                i, &fnote, event_id
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

/// Publish DMs.
pub(crate) async fn send_dms(
    client: &Client,
    notes: &[String],
    recipient: PublicKey,
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
                            // was send_direct_msg: Unsecure! Use `send_private_msg` instead.
                            match client.send_private_msg(recipient, &line, None).await {
                                Ok(event_id) => debug!(
                                    "send_private_msg number {:?} from pipe stream sent successfully. {:?}, sent to {:?}, event_id {:?}",
                                    i, &line, recipient, event_id
                                ),
                                Err(ref e) => {
                                    err_count += 1;
                                    error!(
                                        "send_private_msg number {:?} from pipe stream failed. {:?}, sent to {:?}",
                                        i, &line, recipient
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

        // was send_direct_msg: Unsecure! Use `send_private_msg` instead.
        match client.send_private_msg(recipient, &fnote, None).await {
            Ok(ref event_id) => debug!(
                "DM message number {:?} sent successfully. {:?}, sent to {:?}, event_id {:?}.",
                i, &fnote, recipient, event_id
            ),
            Err(ref e) => {
                err_count += 1;
                error!(
                    "DM message number {:?} failed. {:?}, sent to {:?}.",
                    i, &fnote, recipient
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
            let notes = &ap.dm[1..];
            send_dms(client, notes, pk).await
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

async fn send_channel_message(
    client: &Client,
    channel_id: &PublicKey,
    relay_url: &Url,
    line: &str,
    annotation: &str,
) -> bool {
    let tags: Vec<Tag> = vec![]; // TODO: add relay_url tag
    let event_id = EventId::new(
        &channel_id.clone(),
        &Timestamp::now(),
        &Kind::ChannelMessage,
        &tags,
        line,
    );
    //created_at: &Timestamp,
    //kind: &Kind,
    //tags: &[Tag],
    //content: &str,
    match client
        .send_channel_msg(event_id, relay_url.clone(), line)
        .await
    {
        Ok(ref event_id) => {
            debug!(
                "send_channel_msg number {} sent successfully. {:?}, sent to {:?}, event_id is {:?}",
                annotation, line, &channel_id, event_id
            );
            true
        }
        Err(ref e) => {
            error!(
                "send_channel_msg number {} failed. {:?}, sent to {:?}, error is {:?}",
                annotation, line, &channel_id, e
            );
            false
        }
    }
}

/// Send messages to one channel.
pub(crate) async fn send_channel_messages(
    client: &Client,
    notes: &[String], // msgs
    channel_id: PublicKey,
    relay_url: Url,
) -> Result<(), Error> {
    trace!("send_channel_messages {:?} {:?}.", notes, channel_id);
    let mut err_count = 0usize;
    let num = notes.len();
    let mut i = 0;
    while i < num {
        let note = &notes[i];
        trace!("send_channel_messages: {:?}", note);
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
                            match send_channel_message(
                                client,
                                &channel_id,
                                &relay_url,
                                &line,
                                &format!("{} from pipe stream", i),
                            )
                            .await
                            {
                                false => err_count += 1,
                                _ => (),
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

        match send_channel_message(client, &channel_id, &relay_url, &fnote, &format!("{}", i)).await
        {
            false => err_count += 1,
            _ => (),
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
    // todo: check if hash is valid, doable? check documentation
    match PublicKey::from_str(&ap.send_channel_message[0]) {
        Ok(channel_id) => {
            let notes = &ap.send_channel_message[1..];
            let relay: Url;
            if !ap.relay.is_empty() {
                // todo: pass the vector of relays, not just one
                relay = ap.relay[0].clone();
            } else {
                relay = ap.creds.relays[0].clone().url;
            }
            // todo: using empty relay-vector, should it be set?
            send_channel_messages(client, notes, channel_id, relay).await
        }
        Err(ref e) => {
            error!(
                "Error: Not a valid hash (channel id). Cannot send this channel message. Aborting. hash {:?}, 1st Msg {:?} ",
                ap.send_channel_message[0],
                ap.send_channel_message[1]
            );
            Err(Error::InvalidHash)
        }
    }
}

/// Is key in subscribed_authors list?
pub(crate) fn is_subscribed_author(ap: &Args, pkey: &PublicKey) -> bool {
    ap.creds.subscribed_authors.contains(pkey)
}

/// Get contact for given alias.
/// Returns None if alias does not exist in contact list.
pub(crate) fn get_contact_by_alias(ap: &Args, alias: &str) -> Option<Contact> {
    ap.creds
        .contacts
        .iter()
        .find(|s| s.alias == Some(alias.to_string()))
        .cloned()
}

/// Get contact for given pubkey.
/// Returns None if pubkey does not exist in contact list.
pub(crate) fn get_contact_by_key(ap: &Args, pkey: PublicKey) -> Option<Contact> {
    ap.creds
        .contacts
        .iter()
        .find(|s| s.public_key == pkey)
        .cloned()
}

/// Get contact alias for given pubkey, or if not in contacts return given pubkey.
/// Returns alias if contact with this pubkey exists.
/// Returns input pubkey if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_or_keystr_by_key(ap: &Args, pkey: PublicKey) -> String {
    match get_contact_by_key(ap, pkey) {
        Some(c) => match c.alias {
            Some(a) => a,
            None => pkey.to_string(),
        },
        None => pkey.to_string(),
    }
}

/// Get contact alias for given pubkey, or if not in contacts return None.
/// Returns Some(alias) if contact with this pubkey exists.
/// Returns None if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_by_key(ap: &Args, pkey: PublicKey) -> Option<String> {
    match get_contact_by_key(ap, pkey) {
        Some(c) => c.alias,
        None => None,
    }
}

/// Get contact alias for given pubkey string (string of PublicKey), or if not in contacts return given pubkey.
/// Returns alias if contact with this pubkey exists.
/// Returns input pubkey if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_or_keystr_by_keystr(ap: &Args, pkeystr: &str) -> String {
    match PublicKey::from_str(pkeystr) {
        Ok(pkey) => match get_contact_by_key(ap, pkey) {
            Some(c) => match c.alias {
                Some(a) => a,
                None => pkey.to_string(),
            },
            None => pkey.to_string(),
        },
        Err(_) => pkeystr.to_string(),
    }
}

/// Get contact alias for given pubkey string (string of PublicKey), or if not in contacts return None.
/// Returns Some(alias) if contact with this pubkey exists.
/// Returns None if no contact with this pubkey exists.
pub(crate) fn get_contact_alias_by_keystr(ap: &Args, pkeystr: &str) -> Option<String> {
    match PublicKey::from_str(pkeystr) {
        Ok(pkey) => match get_contact_by_key(ap, pkey) {
            Some(c) => c.alias,
            None => None,
        },
        Err(_) => None,
    }
}

/// Handle the --add-contact CLI argument, write contacts from CLI args into creds data structure
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
                let rurl = ap.relay[i].clone();
                ap.creds.contacts.push(Contact::new(
                    pkey,
                    Some(UncheckedUrl::from(rurl)),
                    Some(ap.alias[i].trim().to_string()),
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

/// Handle the --remove-contact CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_remove_contact(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let num = ap.alias.len();
    let mut i = 0;
    while i < num {
        ap.creds
            .contacts
            .retain(|x| x.alias != Some(ap.alias[i].trim().to_string()));
        i += 1;
    }
    Ok(())
}

/// Convert npub1... Bech32 key or Hex key or contact alias into a PublicKey
/// Returns Error if neither valid Bech32, nor Hex key, nor contact alias.
pub(crate) fn cstr_to_pubkey(ap: &Args, s: &str) -> Result<PublicKey, Error> {
    match get_contact_by_alias(ap, s) {
        Some(c) => Ok(c.public_key),
        None => str_to_pubkey(s),
    }
}

/// Convert npub1... Bech32 key or Hex key into a PublicKey
/// Returns Error if neither valid Bech32 nor Hex key.
pub(crate) fn str_to_pubkey(s: &str) -> Result<PublicKey, Error> {
    match PublicKey::from_bech32(s) {
        Ok(pkey) => {
            debug!(
                "Valid key in Bech32 format: Npub {:?}, Hex {:?}",
                s,
                pkey.to_bech32().unwrap() // public_key
            );
            return Ok(pkey);
        }
        Err(ref e) => match PublicKey::from_str(s) {
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
    match PublicKey::from_bech32(s) {
        Ok(pkey) => {
            debug!(
                "Valid key in Bech32 format: Npub {:?}, Hex {:?}",
                s,
                pkey.to_string() // public_key
            );
            let npub = s.to_owned();
            // todo: zzz not sure about this
            let hex = pkey.to_string(); // pkey.to_bech32().unwrap();
            return Ok((npub, hex));
        }
        Err(ref e) => match PublicKey::from_str(s) {
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

/// Handle the --unsubscribe-pubkey CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_unsubscribe_pubkey(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.unsubscribe_pubkey.len();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.unsubscribe_pubkey[i]) {
            Ok(pkey) => {
                ap.creds.subscribed_pubkeys.retain(|x| x != &pkey);
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not removed from subscription filter.",
                    &ap.unsubscribe_pubkey[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::UnsubscribeFailed)
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

/// Handle the --unsubscribe-author CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_unsubscribe_author(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.unsubscribe_author.len();
    let mut i = 0;
    while i < num {
        match str_to_pubkey(&ap.unsubscribe_author[i]) {
            Ok(pkey) => {
                ap.creds.subscribed_authors.retain(|x| x != &pkey);
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not removed from subscription filter.",
                    &ap.unsubscribe_author[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::UnsubscribeFailed)
    } else {
        Ok(())
    }
}

/// Handle the --subscribe-channel CLI argument, moving pkeys from CLI args into creds data structure
pub(crate) async fn cli_subscribe_channel(client: &mut Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.subscribe_channel.len();
    let mut hashs = Vec::new();
    let mut i = 0;
    while i < num {
        match PublicKey::from_str(&ap.subscribe_channel[i]) {
            Ok(hash) => {
                hashs.push(hash);
                debug!(
                    "Valid key added to subscription filter. Key {:?}, hash: {:?}.",
                    &ap.subscribe_channel[i],
                    hash.to_string()
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
    ap.creds.subscribed_channels.append(&mut hashs);
    ap.creds.subscribed_channels.dedup_by(|a, b| a == b);
    if err_count != 0 {
        Err(Error::SubscriptionFailed)
    } else {
        Ok(())
    }
}

/// Handle the --unsubscribe-channel CLI argument, remove CLI args contacts from creds data structure
pub(crate) async fn cli_unsubscribe_channel(client: &Client, ap: &mut Args) -> Result<(), Error> {
    let mut err_count = 0usize;
    let num = ap.unsubscribe_channel.len();
    let mut i = 0;
    while i < num {
        match PublicKey::from_str(&ap.unsubscribe_channel[i]) {
            Ok(hash) => {
                ap.creds.subscribed_channels.retain(|x| x != &hash);
                debug!(
                    "Valid key removed from subscription filter. Key {:?}, hash: {:?}.",
                    &ap.unsubscribe_channel[i],
                    hash.to_string()
                );
            }
            Err(ref e) => {
                error!(
                    "Error: Invalid key {:?}. Not removed from subscription filter.",
                    &ap.unsubscribe_channel[i]
                );
                err_count += 1;
            }
        }
        i += 1;
    }
    if err_count != 0 {
        Err(Error::UnsubscribeFailed)
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
    let mut is_connected = false; // is this app connected to relays

    eprintln!("If you know Rust a bit, if you are interested in Nostr, ");
    eprintln!("then please consider making a code contribution. ");
    eprintln!("At the very least give it a star on Github. ");
    eprintln!("Star and make PRs at: https://github.com/8go/nostr-commander-rs ");
    eprintln!("");

    // handle log level and debug options
    let env_org_rust_log = env::var("RUST_LOG").unwrap_or_default().to_uppercase();
    // println!("Original log_level option is {:?}", ap.log_level);
    // println!("Original RUST_LOG is {:?}", &env_org_rust_log);
    match ap.debug.cmp(&1) {
        Ordering::Equal => {
            // -d overwrites --log-level
            let llvec = vec![LogLevel::Debug];
            ap.log_level = Some(llvec);
        }
        Ordering::Greater => {
            // -d overwrites --log-level
            let mut llvec = vec![LogLevel::Debug];
            llvec.push(LogLevel::Debug);
            ap.log_level = Some(llvec);
        }
        Ordering::Less => (),
    }
    match ap.log_level.clone() {
        None => {
            tracing_subscriber::fmt()
                .with_writer(io::stderr)
                .with_env_filter(EnvFilter::from_default_env()) // support the standard RUST_LOG env variable
                .init();
            debug!("Neither --debug nor --log-level was used. Using environment vaiable RUST_LOG.");
        }
        Some(llvec) => {
            if llvec.len() == 1 {
                if llvec[0].is_none() {
                    return Err(Error::UnsupportedCliParameter(
                        "Value 'none' not allowed for --log-level argument",
                    ));
                }
                // .with_env_filter("matrix_commander_rs=debug") // only set matrix_commander_rs
                let mut rlogstr: String = BIN_NAME_UNDERSCORE.to_owned();
                rlogstr.push('='); // add char
                rlogstr.push_str(&llvec[0].to_string());
                tracing_subscriber::fmt()
                    .with_writer(io::stderr)
                    .with_env_filter(rlogstr.clone()) // support the standard RUST_LOG env variable for default value
                    .init();
                debug!(
                    "The --debug or --log-level was used once or with one value. \
                    Specifying logging equivalent to RUST_LOG seting of '{}'.",
                    rlogstr
                );
            } else {
                if llvec[0].is_none() || llvec[1].is_none() {
                    return Err(Error::UnsupportedCliParameter(
                        "Value 'none' not allowed for --log-level argument",
                    ));
                }
                // RUST_LOG="error,matrix_commander_rs=debug"  .. This will only show matrix-comander-rs
                // debug info, and erors for all other modules
                let mut rlogstr: String = llvec[1].to_string().to_owned();
                rlogstr.push(','); // add char
                rlogstr.push_str(BIN_NAME_UNDERSCORE);
                rlogstr.push('=');
                rlogstr.push_str(&llvec[0].to_string());
                tracing_subscriber::fmt()
                    .with_writer(io::stderr)
                    .with_env_filter(rlogstr.clone())
                    .init();
                debug!(
                    "The --debug or --log-level was used twice or with two values. \
                    Specifying logging equivalent to RUST_LOG seting of '{}'.",
                    rlogstr
                );
            }
            if llvec.len() > 2 {
                debug!("The --log-level option was incorrectly used more than twice. Ignoring third and further use.")
            }
        }
    }
    if ap.debug > 0 {
        info!("The --debug option overwrote the --log-level option.")
    }
    if ap.debug > 2 {
        debug!("The --debug option was incorrectly used more than twice. Ignoring third and further use.")
    }
    debug!("Original RUST_LOG env var is '{}'", env_org_rust_log);
    debug!(
        "Final RUST_LOG env var is '{}'",
        env::var("RUST_LOG").unwrap_or_default().to_uppercase()
    );
    debug!("Final log-level option is {:?}", ap.log_level);
    if enabled!(Level::TRACE) {
        debug!(
            "Log level of module {} is set to TRACE.",
            get_prog_without_ext()
        );
    } else if enabled!(Level::DEBUG) {
        debug!(
            "Log level of module {} is set to DEBUG.",
            get_prog_without_ext()
        );
    }
    debug!("Version is {}", get_version());
    debug!("Package name is {}", get_pkg_name());
    debug!("Repo is {}", get_pkg_repository());
    debug!("Arguments are {:?}", ap);

    match ap.version {
        None => (),                        // do nothing
        Some(None) => crate::version(&ap), // print version
        Some(Some(Version::Check)) => crate::version_check(&ap),
    }
    if ap.contribute {
        crate::contribute(&ap);
    };
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
                info!("User credentials read successfully.");
            }
            Err(ref e) => {
                if ap.nsec.is_some() {
                    debug!("User id will be taken from --nsec argument.");
                } else {
                    error!("Credentials file does not exists or cannot be read. Try creating a user first with --create-user. Check your arguments and try again. Worst case if file is corrupted or lost, consider doing a '--delete-user' to clean up, then perform a new '--create-user'. {:?}.", e);
                    return Err(Error::ReadingCredentialsFailed);
                }
            }
        }
    }
    if ap.nsec.is_some() {
        debug!("We use private and public key from --nsec argument.");
        // parses from bech32 as well as from hex
        let my_keys = Keys::parse(&ap.nsec.clone().unwrap())?;
        ap.creds.public_key_bech32 = my_keys.public_key().to_bech32().unwrap();
        ap.creds.secret_key_bech32 = my_keys.secret_key().to_bech32().unwrap();
    }
    // credentials are filled now

    debug!("Welcome to nostr-commander-rs");

    let my_keys = Keys::parse(&ap.creds.secret_key_bech32)?;

    // Show public key
    if ap.show_public_key {
        debug!(
            "Loaded public key in Nostr format is : {:?}",
            my_keys.public_key().to_string()
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
            my_keys.secret_key().display_secret()
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

    match add_relays_from_creds(&mut client, &mut ap).await {
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
        match crate::cli_add_relay(&mut client, &mut ap).await {
            Ok(()) => {
                info!("add_relay successful.");
            }
            Err(ref e) => {
                error!("add_relay failed. Reported error is: {:?}", e);
            }
        }
    }
    if !ap.remove_relay.is_empty() {
        match crate::cli_remove_relay(&client, &mut ap).await {
            Ok(()) => {
                info!("remove_relay successful.");
            }
            Err(ref e) => {
                error!("remove_relay failed. Reported error is: {:?}", e);
            }
        }
    }
    ap.creds.relays.dedup_by(|a, b| a.url == b.url);

    trace!("checking to see if it is necessary to call connect.");
    // todo: further optimize: --unsubscribe-... could remove subscriptions and make subscriptions empty,
    // but this is not yet checked.
    if ap.listen
        // || !ap.publish_pow.is_empty() // publish_pow_text_note discontinued since nostr-sdk v0.21.
        || !ap.publish.is_empty()
        || !ap.dm.is_empty()
        || !ap.send_channel_message.is_empty()
        || !ap.subscribe_pubkey.is_empty()
        || !ap.subscribe_author.is_empty()
        || !ap.subscribe_channel.is_empty()
    {
        // design decision: avoid connect_...()  call if no relay action is needed and everything can be done locally.
        // design decision: avoid connect...() if no client is needed.
        //
        // Do we need to connect on create-user ? No. --create-user just creates locally a key-pair.
        info!("initiating connect now.");
        client.connect().await;
        info!("connect successful.");
        if client.relays().await.is_empty() {
            error!("Client has no relay. Certain operations will fail. Consider using --add-relay argument.")
        }
        is_connected = true;
    }

    if ap.create_user {
        // let metadata = Metadata::new()
        //     .name("username")
        //     .display_name("My Username")
        //     .about("Description")
        //     .picture(Url::from_str("https://example.com/avatar.png")?)
        //     .nip05("username@example.com");

        // Update profile metadata
        // client.update_profile() was removed from nostr-sdk API
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
    if is_connected {
        trace!("setting contact list on server.");
        match client.set_contact_list(ap.creds.contacts.clone()).await {
            Ok(ref event_id) => {
                info!("set_contact_list successful. event_id {:?}", event_id);
            }
            Err(ref e) => {
                error!("set_contact_list failed. Reported error is: {:?}", e);
            }
        }
    } else {
        trace!("not setting contact list on server, because we are not connected.");
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

    trace!("checking if something needs to be published.");
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
    // publish_pow_text_note discontinued since nostr-sdk v0.21.
    // // Publish a POW text note
    // if !ap.publish_pow.is_empty() {
    //     match crate::cli_publish_pow(&client, &mut ap).await {
    //         Ok(()) => {
    //             info!("publish_pow successful.");
    //         }
    //         Err(ref e) => {
    //             error!("publish_pow failed. Reported error is: {:?}", e);
    //         }
    //     }
    // }
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
        let filter = Filter::new().pubkeys(ap.creds.subscribed_pubkeys.clone());
        subscribe_to_filter(&client, &ap, filter, "keys").await;
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
        let filter = Filter::new().authors(ap.creds.subscribed_authors.clone());
        subscribe_to_filter(&client, &ap, filter, "authors").await;
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
    // Unsubscribe channels
    if !ap.unsubscribe_channel.is_empty() {
        match crate::cli_unsubscribe_channel(&mut client, &mut ap).await {
            Ok(()) => {
                debug!("unsubscribe_channel successful. Subscriptions synchronized with credentials file.");
            }
            Err(ref e) => {
                error!("unsubscribe_channel failed. Reported error is: {:?}", e);
            }
        }
    }
    if !ap.creds.subscribed_channels.is_empty() && ap.listen {
        let filter = Filter::new().pubkeys(ap.creds.subscribed_channels.clone());
        subscribe_to_filter(&client, &ap, filter, "channels").await;
    }
    if ap.nsec.is_none() || ap.create_user {
        debug!("Creating or updating credentials file.");
        ap.creds.save(get_credentials_actual_path(&ap))?;
    } else {
        debug!("Not creating or not updating credentials file.")
    }

    // notices will be published even if we do not go into handle_notification event loop
    // Design choice: Do not automatically listen when subscriptions exist, only listen to subscriptions if --listen is set.
    if ap.listen
    // || !ap.creds.subscribed_authors.is_empty()
    // || !ap.creds.subscribed_pubkeys.is_empty()
    {
        let mut num = ap.publish.len() + ap.dm.len() + ap.send_channel_message.len(); // + ap.publish_pow.len() // publish_pow_text_note discontinued since nostr-sdk v0.21.
        if ap.dm.len() > 1 {
            num -= 1; //adjust num, 1st arg of dm is key not msg
        }
        if ap.send_channel_message.len() > 1 {
            num -= 1; //adjust num, 1st arg of send_channel_message is key not msg
        }
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
            .handle_notifications(|notification| async {
                debug!("Notification: {:?}", notification);
                match notification {
                    RelayPoolNotification::Authenticated { relay_url } => {
                        debug!("Relay Authenticated: relay url {:?}", relay_url);
                        // todo!()
                    }
                    RelayPoolNotification::Shutdown => {
                        debug!("Shutdown: shutting down");
                        // todo: zzz shutdown
                    }
                    RelayStatus { relay_url, status } => {
                        debug!("Event-RelayStatus: url {:?}, relaystatus {:?}", relay_url, status);
                    }
                    Event { relay_url, subscription_id, event } => {
                        debug!("Event-Event: url {:?}, content {:?}, kind {:?}", relay_url, event.content, event.kind);
                    }
                    Message {relay_url, message } => {
                        // debug!("Message: {:?}", message);
                        match message {
                            RelayMessage::Ok {event_id, status, message } => {
                                // Notification: ReceivedMessage(Ok { event_id: 123, status: true, message: "" })
                                // confirmation of notice having been relayed
                                info!(concat!(
                                        r#"Message-OK: Notice, DM or message was relayed. Url is {:?}"#,
                                        r#" Event id is {:?}. Status is {:?} and message is {:?}. You"#,
                                        r#" can investigate this event by looking it up on https://nostr.com/e/{}"#
                                      ),
                                      relay_url, event_id, status, message, event_id.to_string()
                                );
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
                                let first = true;
                                for t in &event.tags {
                                    match t.kind() {
                                        TagKind::SingleLetter(p) => {
                                            trace!("tag vector: {:?}", t.as_slice());
                                            //match t.content() {
                                            //    Some(c) => {
                                            //        trace!("tag: {:?}", get_contact_alias_or_keystr_by_keystr(&ap, c));
                                            //        match get_contact_alias_by_keystr(&ap, c) {
                                            //            Some(a) => {
                                            //                if !first { tags += ", "; };
                                            //                tags += &a;
                                            //                first = false;
                                            //                },
                                            //            _ => ()
                                            //        }
                                            //    }
                                            //    None => ()
                                            //}
                                        },
                                        // TagKind::SingleLetter(e) => info!("Single letter message received. Not implemented. {:?}",e),  // todo!(),
                                        TagKind::Nonce => info!("Nonce message received. Not implemented."),  // todo!(),
                                        TagKind::Delegation => info!("Delegation message received. Not implemented."),  // todo!(),
                                        TagKind::ContentWarning => info!("ContentWarning message received. Not implemented."),  // todo!(),
                                        TagKind::Custom(_) => info!("Custom message received. Not implemented."),  // todo!(),
                                        _ => info!("Other message received. Not implemented."),  // todo!(),
                                    }
                                }
                                trace!("Message-Event: content {:?}, kind {:?}, from pubkey {:?}, with tags {:?}", event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey), event.tags);
                                let mut key_author = "key";
                                if is_subscribed_author(&ap, &event.pubkey) {
                                            key_author = "author";
                                            tags = get_contact_alias_or_keystr_by_key(&ap, event.pubkey);
                                        };
                                match event.kind {
                                    Kind::ContactList => {
                                        debug!("Received Message-Event ContactList");
                                    },
                                    Kind::Reaction => {
                                        debug!("Received Message-Event Reaction: content {:?}", event.content);
                                    },
                                    Kind::TextNote => {
                                        info!("Subscription by {} ({}): content {:?}, kind {:?}, from pubkey {:?}", key_author, tags, event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey));
                                        print_json(
                                            &json!({
                                                "event_type": "RelayMessage::Event",
                                                "event_type_meaning": "Message was received because of subscription.",
                                                "subscribed_by": key_author,
                                                "author": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "content": event.content,
                                                "kind": event.kind, // writes integer like '1'
                                                "kind_text": format!("{:?}",event.kind), // writes text like "Base(TextNote)"
                                                "from_alias": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "from_pubkey": event.pubkey,
                                                "tags": tags
                                            }) ,
                                            ap.output,0,""
                                        );
                                    },
                                    Kind::ChannelMessage => {
                                        info!("Subscription by {} ({}): content {:?}, kind {:?}, from pubkey {:?}", key_author, tags, event.content, event.kind, get_contact_alias_or_keystr_by_key(&ap, event.pubkey));
                                        print_json(
                                            &json!({
                                                "event_type": "RelayMessage::Event",
                                                "event_type_meaning": "Message was received because of subscription.",
                                                "subscribed_by": key_author,
                                                "author": get_contact_alias_or_keystr_by_key(&ap, event.pubkey),
                                                "content": event.content,
                                                "kind": event.kind, // writes integer like '1'
                                                "kind_text": format!("{:?}",event.kind), // writes text like "Base(TextNote)"
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
                            RelayMessage::EndOfStoredEvents(subscription_id) =>  {
                                debug!("Received Message-Event EndOfStoredEvents");
                            },
                            RelayMessage::Auth { challenge } =>  {
                                debug!("Received Message-Event Auth");
                            },
                            RelayMessage::Count { subscription_id, count } =>  {
                                debug!("Received Message-Event Count {:?}", count);
                            },
                            RelayMessage::Closed { subscription_id, message } => {
                                debug!("Received Message-Event Closed {:?}", message);
                            },
                            RelayMessage::NegMsg { subscription_id, message } => {
                                debug!("Received Message-Event NegMsg {:?}", message);
                            },
                            RelayMessage::NegErr { subscription_id, code } => {
                                debug!("Received Message-Event NegErr {:?}", code);
                            },
                        }
                    }
                }
                Ok(false)
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

async fn subscribe_to_filter(client: &Client, ap: &Args, mut filter: Filter, filter_name: &str) {
    if ap.limit_number != 0 {
        filter = filter.limit(ap.limit_number);
    }
    if ap.limit_days != 0 {
        filter = filter.since(Timestamp::now() - Duration::new(ap.limit_days * 24 * 60 * 60, 0));
    }
    if ap.limit_hours != 0 {
        filter = filter.since(Timestamp::now() - Duration::new(ap.limit_hours * 60 * 60, 0));
    }
    if ap.limit_future_days != 0 {
        filter =
            filter.until(Timestamp::now() + Duration::new(ap.limit_future_days * 24 * 60 * 60, 0));
    }
    if ap.limit_future_hours != 0 {
        filter = filter.until(Timestamp::now() + Duration::new(ap.limit_future_hours * 60 * 60, 0));
    }
    info!("subscribe to {filter_name} initiated.");
    match client.subscribe(vec![filter], None).await {
        Ok(..) => info!("subscribe to {filter_name} successful."),
        Err(ref e) => error!("handle_notifications failed. Reported error is: {:?}", e),
    }
}

/// Future test cases will be put here
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_usage() {
        assert_eq!(usage(), ());
    }

    #[test]
    fn test_help() {
        assert_eq!(help(), ());
    }
}
