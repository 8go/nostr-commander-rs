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
//! - nostr-commander-rs --create-user --name "James Jones" \
//!     --display-name Jimmy --about "tech and pizza lover" \
//!     --picture "https://i.imgur.com/mIcObyL.jpeg" \
//!     --nip05 jim@nostr.example.org \
//!     --add-relay "wss://nostr.openchain.fr" "wss://relay.damus.io" # first time only
//! - nostr-commander-rs --publish "Love this protocol"
//! - nostr-commander-rs --dm joe@example.org "How about pizza tonight?"
//!
//! For more information, see the README.md
//! <https://github.com/8go/nostr-commander-rs/blob/main/README.md>
//! file.

// #![allow(dead_code)] // crate-level allow  // Todo
#![allow(unused_variables)] // Todo
#![allow(unused_imports)] // Todo

// use atty::Stream;
use clap::{ColorChoice, Parser, ValueEnum};
use directories::ProjectDirs;
// use mime::Mime;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{self, Debug};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use tracing::{debug, enabled, error, info, warn, Level};
use tracing_subscriber;
use update_informer::{registry, Check};
use url::Url;

use nostr_sdk::{
    nostr::key::{FromBech32, KeyError, Keys, ToBech32},
    nostr::message::relay::RelayMessage,
    // nostr::util::nips::nip04::Error as Nip04Error,
    nostr::Metadata,
    relay::pool::RelayPoolNotifications,
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

    #[error("Send Failed")]
    SendFailed,

    #[error("Listen Failed")]
    ListenFailed,

    #[error("Invalid Client Connection")]
    InvalidClientConnection,

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
    // pub fn is_text(&self) -> bool {
    //     self == &Self::Text
    // }

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
#[command(author, version, next_line_help = true,
    bin_name = get_prog_without_ext(),
    color = ColorChoice::Always,
    term_width = 79,
    after_help = "",
    disable_version_flag = true,
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
    // Todo: add '-' to read from stdin or keyboard
    #[arg(short, long, value_name = "NOTE", num_args(0..), )]
    publish: Vec<String>,

    /// Publish one or multiple notes with proof-of-work (POW).
    /// Use also '--pow-difficulty' to specify difficulty.
    // Todo: add '-' to read from stdin or keyboard
    #[arg(long, alias = "pow", value_name = "NOTE", num_args(0..), )]
    publish_pow: Vec<String>,

    /// Send one or multiple DMs. DM messages will be encrypted and
    /// preserve privacy.
    // Todo: add '-' to read from stdin or keyboard
    #[arg(long, alias = "direct", value_name = "NOTE", num_args(0..), )]
    dm: Vec<String>,

    /// Add one or multiple relays. A relay is specified via a URI
    /// that looks like 'wss://some.relay.org'. You can find relays
    /// by looking at https://github.com/aljazceru/awesome-nostr#instances.
    /// Sampler relay registries are: https://nostr-registry.netlify.app/,
    /// https://nostr.info/, or https://nostr.watch/.
    /// Examples: "wss://relay.damus.io", "wss://nostr.openchain.fr".
    /// See also '--proxy'.
    #[arg(long, value_name = "RELAY_URI", num_args(0..), )]
    add_relay: Vec<String>,

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

    /// Show public key..
    #[arg(long, default_value_t = false)]
    show_public_key: bool,

    /// Show private, secret key. Protect this key.
    #[arg(long, default_value_t = false)]
    show_secret_key: bool,

    /// Print the user name used by "matrix-commander-rs" (itself).
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
    /// Control-C on your keyboard. E.g. this helps you get event ids for
    /// published notices.
    #[arg(short, long, default_value_t = false)]
    listen: bool,
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
            contribute: false,
            version: None,
            debug: 0u8,
            log_level: LogLevel::None,
            verbose: 0u8,
            // plain: false,
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

/// Handle the --publish CLI argument
pub(crate) async fn cli_publish(client: &Client, ap: &mut Args) -> Result<(), Error> {
    // Todo
    // Publish notes
    let num = ap.publish.len();
    let mut i = 0;
    while i < num {
        client.publish_text_note(&ap.publish[i], &[]).await?;
        i += 1;
    }
    Ok(())
}

/// Handle the --publish_pow CLI argument
pub(crate) async fn cli_publish_pow(client: &Client, ap: &mut Args) -> Result<(), Error> {
    // Todo
    // Publish a POW text note
    let num = ap.publish.len();
    let mut i = 0;
    while i < num {
        client
            .publish_pow_text_note(&ap.publish[i], &[], ap.pow_difficulty)
            .await?;
        i += 1;
    }
    Ok(())
}

/// Utility function to print JSON object as JSON or as plain text
pub(crate) fn print_json(json_data: &json::JsonValue, output: Output) {
    debug!("{:?}", json_data);
    match output {
        Output::Text => {
            let mut first = true;
            for (key, val) in json_data.entries() {
                if first {
                    first = false;
                } else {
                    print!("    ");
                }
                print!("{}:", key);
                if val.is_object() {
                    // if it is an object, check recursively
                    print_json(val, output);
                } else if val.is_boolean() {
                    print!("    {}", val);
                } else if val.is_null() {
                    print!("    "); // print nothing
                } else if val.is_string() {
                    print!("    {}", val);
                } else if val.is_number() {
                    print!("    {}", val);
                } else if val.is_array() {
                    print!("    [{}]", val);
                }
            }
            println!();
        }
        Output::JsonSpec => (),
        _ => {
            println!("{}", json_data.dump(),);
        }
    }
}

/// Handle the --whoami CLI argument
pub(crate) fn cli_whoami(ap: &Args) -> Result<(), Error> {
    print_json(
        &json::object!(
            name: ap.creds.metadata.name.clone(),
            display_name: ap.creds.metadata.display_name.clone(),
        ),
        ap.output,
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

    // Connect to relays, WAIT for connection, and keep connection alive
    // match client.connect().await {
    match client.connect_and_wait().await {
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

    // notices will be published even if we do not go into handle_notification event loop
    if ap.listen {
        let num = ap.publish.len() + ap.publish_pow.len();
        info!(
            "You should be receiving {:?} 'OK' messages with event ids, one for each notice that has been relayed.",
            num
        );
        // Handle notifications
        match client
            .handle_notifications(|notification| {
                debug!("Notification: {:?}", notification);
                match notification {
                    ReceivedEvent(ev) => {
                        debug!("Event: {:?}", ev);
                    }
                    ReceivedMessage(msg) => {
                        debug!("Message: {:?}", msg);
                        // Notification: ReceivedMessage(Ok { event_id: 123, status: true, message: "" })
                        // confirmation of notice having been relayed
                        match msg {
                            RelayMessage::Ok {event_id, status, message } => {
                                println!("OK: Notice was relayed. Event id is {:?}. Status is {:?} and message is {:?}. You can investigate this event by looking it up on https://nostr.com/e/{}", event_id, status, message, event_id.to_string());
                            },
                            RelayMessage::Notice { message } => {
                                debug!("Notice: {:?}", message);
                            }
                            RelayMessage::Empty => (),
                            _ => (),
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
