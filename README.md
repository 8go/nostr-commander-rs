[![crates.io - Version](
https://img.shields.io/crates/v/nostr-commander
)](https://crates.io/crates/nostr-commander)
[![crates.io - Downloads](
https://img.shields.io/crates/d/nostr-commander
)](https://crates.io/crates/nostr-commander)

<p>
<img
src="https://raw.githubusercontent.com/8go/nostr-commander-rs/master/logos/nostr-commander-rs.svg"
alt="NC logo" height="150">

# nostr-commander-rs

TLDR: simple but convenient CLI-based Nostr client app for publishing, sending DMs, as well as following users and channels

nostr-commander is a simple terminal-based CLI client of
Nostr <https://github.com/nostr-protocol/nostr> written in Rust. 
It lets you create a
Nostr user, subscribe and follow posts of other
users, send private (encrypted) DMs to your Nostr friends,
and much more.

Please help improve the code and add features  :pray:  :clap:
Any contribution is welcome.

Please give it a star :star: right away on Github so that other people
can find the project more easily :heart:.

# What's in the name?

nostr-command*(lin)*er. A word play.

# Audience, Use cases

- for terminal lovers
- for admins, for automating tasks
- for writing shell scripts to interact with the `Nostr` network

# Links

- https://github.com/8go/nostr-commander-rs
- https://crates.io/crates/nostr-commander
- https://docs.rs/crate/nostr-commander

# Installation

- Go to Github releases
- Build from source
    - Install the Rust compiler and Cargo using `rustup.rs`. See https://rustup.rs/.
    - Clone this repository (this command will clone to your home directory):
        - `git clone https://github.com/8go/nostr-commander-rs.git ~/nostr-commander-rs
    - Change directory into repository directory
        - `cd ~/nostr-commander-rs`
    - Run the build command with the release flag
        - `cargo build --release`
    - Once program is compiled, the executable will be available in target/release/nostr-commander-rs.
        - `./target/release/nostr-commander-rs --version # run it and get version`

# Example Usage

```
$ nostr-commander-rs --create-user --name "James Jones" \
    --display-name Jimmy --about "tech and pizza lover" \
    --picture "https://i.imgur.com/mIcObyL.jpeg" \
    --nip05 jim@nostr.example.org \
    --add-relay "wss://nostr.openchain.fr" "wss://relay.damus.io" # first time only
$ nostr-commander-rs --publish "Love this protocol"
$ nostr-commander-rs --dm joe "How about pizza tonight?"
```

# Usage

```
Welcome to "nostr-commander-rs", a Nostr CLI client. ─── On the first run use
--create-user to create a user. On further runs you can publish notes, send
private DM messages, etc.  ─── Have a look at the repo
"https://github.com/8go/nostr-commander-rs/" and see if you can contribute code
to improve this tool. Safe!

Usage: nostr-commander-rs [OPTIONS]

Options:
      --contribute
          Please contribute
  -v, --version [<CHECK>]
          Print version number or check if a newer version exists on crates.io.
          If used without an argument such as '--version' it will print the
          version number. If 'check' is added ('--version check') then the
          program connects to https://crates.io and gets the version number of
          latest stable release. There is no "calling home" on every run, only
          a "check crates.io" upon request. Your privacy is protected. New
          release is neither downloaded, nor installed. It just informs you [possible
          values: check]
  -d, --debug...
          Overwrite the default log level. If not used, then the default log
          level set with environment variable 'RUST_LOG' will be used. If used,
          log level will be set to 'DEBUG' and debugging information will be
          printed. '-d' is a shortcut for '--log-level DEBUG'. See also
          '--log-level'. '-d' takes precedence over '--log-level'.
          Additionally, have a look also at the option '--verbose'
      --log-level <LOG_LEVEL>
          Set the log level by overwriting the default log level. If not used,
          then the default log level set with environment variable 'RUST_LOG'
          will be used. See also '--debug' and '--verbose' [default: none]
          [possible values: none, error, warn, info, debug, trace]
      --verbose...
          Set the verbosity level. If not used, then verbosity will be set to
          low. If used once, verbosity will be high. If used more than once,
          verbosity will be very high. Verbosity only affects the debug
          information. So, if '--debug' is not used then '--verbose' will be
          ignored
  -c, --credentials <PATH_TO_FILE>
          Path to a file containing credentials. At --create-user, information
          about the user, in particular its keys, will be written to a
          credentials file. By default, this file is "credentials.json". On
          further runs the credentials file is read to permit acting as this
          established Nostr user. If this option is provided, the provided path
          to a file will be used as credentials file instead of the default one [default:
          /home/user/.local/share/nostr-commander-rs/credentials.json]
      --create-user
          Create a new user, i.e. a new key pair. This is usually done only
          once at the beginning. If you ever want to wipe this user, use
          '--delete-user' which deletes the key pair. Use this option in
          combination with --name, --display_name, --about, --picture, and
          --nip05. Also highly recommended that you use this option together
          with --add-relay
      --delete-user
          Delete the current user, i.e. delete the current key pair. This will
          erase the key pair and other associated information like user name,
          display name, etc. Afterwards one can create a new user with
          '--create-user'
      --name <USER_NAME>
          Used this to specify an optional user name. Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''
      --display-name <DISPLAY_NAME>
          Used this to specify an optional display name. Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''
      --about <DESCRIPTION>
          Used this to specify an optional description. Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''
      --picture <URL>
          Used this to specify an optional picture or avatar. Used together
          with '--create-user'. Provide a URL like
          'https://example.com/avatar.png'. If this option is not set during
          '--create-user', the information will be queried via the keyboard. If
          you want to set it to empty and not be queried, provide this URL
          'none:'
      --nip05 <NIP05_ID>
          Used this to specify an optional nip05 name. Used together with
          '--create-user'. Provide a nip05 name like 'john@example.org'. If
          this option is not set during '--create-user', the information will
          be queried via the keyboard. If you want to set it to empty and not
          be queried, provide an empty string ''
  -p, --publish [<NOTE>...]
          Publish one or multiple notes
      --publish-pow [<NOTE>...]
          Publish one or multiple notes with proof-of-work (POW). Use also
          '--pow-difficulty' to specify difficulty
      --dm [<KEY+MSGS>...]
          Send one or multiple DMs to one given user. DM messages will be
          encrypted and preserve privacy. The single recipient is specified via
          its public key, a string in the form of 'npub1...', a Hex key, or an
          alias from one of your contacts. The first argument is the recipient,
          all further arguments are texts to be sent. E.g. '-dm
          'npub1SomeStrangeNumbers "First msg" "Second msg"' or 'dm joe "How
          about pizza tonight?"'
      --add-relay [<RELAY_URI>...]
          Add one or multiple relays. A relay is specified via a URI that looks
          like 'wss://some.relay.org'. You can find relays by looking at
          https://github.com/aljazceru/awesome-nostr#instances. Sampler relay
          registries are: https://nostr-registry.netlify.app/,
          https://nostr.info/, or https://nostr.watch/. Examples:
          "wss://relay.damus.io", "wss://nostr.openchain.fr". See also
          '--proxy'
      --show-metadata
          Display current metadata
      --change-metadata
          Modify existing metadata of the user. Use this option in combination
          with --name, --display_name, --about, --picture, and --nip05
      --pow-difficulty <DIFFICULTY>
          Optional proof-of-work (POW) difficulty. Use with '--publish_pow' to
          specify difficulty. If not specified the default will be used [default:
          20]
      --proxy <PROXY>
          Specify a proxy. Used by --add-relay
      --show-public-key
          Show public key
      --show-secret-key
          Show private, secret key. Protect this key
      --whoami
          Print the user name used by "nostr-commander-rs". One can get this
          information also by looking at the credentials file or by using
          --show-metadata
  -o, --output <OUTPUT_FORMAT>
          This option decides on how the output is presented. Currently offered
          choices are: 'text', 'json', 'json-max', and 'json-spec'. Provide one
          of these choices. The default is 'text'. If you want to use the
          default, then there is no need to use this option. If you have chosen
          'text', the output will be formatted with the intention to be
          consumed by humans, i.e. readable text. If you have chosen 'json',
          the output will be formatted as JSON. The content of the JSON object
          matches the data provided by the nostr-sdk SDK. In some occassions
          the output is enhanced by having a few extra data items added for
          convenience. In most cases the output will be processed by other
          programs rather than read by humans. Option 'json-max' is practically
          the same as 'json', but yet another additional field is added. In
          most cases the output will be processed by other programs rather than
          read by humans. Option 'json-spec' only prints information that
          adheres 1-to-1 to the Nostr Specification. Currently this type is not
          supported. If no data is available that corresponds exactly with the
          Nostr Specification, no data will be printed [default: text]
          [possible values: text, json, json-max, json-spec]
  -l, --listen
          Listen to events, notifications and messages. This option listens to
          events and messages forever. To stop, type Control-C on your
          keyboard. You want to listen if you want to get the event ids for
          published notices. Subscriptions do not automatically turn listening
          on. If you want to listen to your subscriptions, you must use
          --listen
      --add-contact
          Add one or more contacts. Must be used in combination with --alias,
          --key, --relay. If you want to add N new contacts, use --add-contact
          and provide exactly N entries in each of the 3 extra arguments. E.g.
          --add-contact --alias jane joe --key npub1JanesPublicKey
          npub1JoesPublicKey --relay "wss://janes.relay.org"
          "wss://joes.relay.org". Aliases must be unique. Alias can be seen as
          a nickname
      --remove-contact
          Remove one or more contacts. Must be used in combination with
          --alias. For each entry in --alias the corresponding contact will be
          removed. E.g. --remove-contact --alias jane joe
      --show-contacts
          Display current contacts
      --alias [<ALIAS>...]
          Provide one or multiple aliases (nicknames) for arguments
          --add-contact and --remove-contact
      --key [<KEY>...]
          Provide one or multiple public keys for argument --add-contact. They
          have the form 'npub1SomeStrangeString'
      --relay [<RELAY>...]
          Provide one or multiple relays for argument --add-contact. They have
          the form 'wss://some.relay.org'
      --subscribe-author [<KEY>...]
          Subscribe to one or more authors. Specify each author by its public
          key in form of 'npub1SomePublicKey'. Alternatively you can use the
          Hex form of the private key
      --subscribe-pubkey [<KEY>...]
          Subscribe to one or more public keys. Specify each public key in form
          of 'npub1SomePublicKey'. Alternatively you can use the Hex form of
          the private key
  -h, --help
          Print help information (use `--help` for more detail)
```
