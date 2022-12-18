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

nostr-command(lin)er. A word play.

# Audience, Use cases

- for terminal lovers
- for admins, for automating tasks
- for writing shell scripts to interact with the `Nostr` network

# Links

- https://github.com/8go/nostr-commander-rs
- https://crates.io/crates/nostr-commander
- https://docs.rs/crate/nostr-commander
- https://github.com/yukibtc/nostr-rs-sdk: `nostr-sdk` used to build `nostr-commander`

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

# Config File

You don't need to know any of this. This is just for the curious ones.

The config file looks something like this. If you want to do some quick testing, 
you can copy and paste this config file to get going real fast.

```
{
  "secret_key_bech32": "nsec1yljk9us0e3whjnzysu6pqjhnw5wglkr6hvx4vj376fs0sfaxze6qvx5f5x",
  "public_key_bech32": "npub1af7ep6s5esrgtc2c7tlvd3v4jpna44qf6nhan8tek6h505nwrvgq38nwz6",
  "relays": [
    "wss://nostr-pub.wellorder.net/"
  ],
  "metadata": {
    "name": "James Jones",
    "display_name": "Jim",
    "about": "tech nerd and nostr lover"
  },
  "contacts": [
    {
      "pk": "887645fef0ce0c3c1218d2f5d8e6132a19304cdc57cd20281d082f38cfea0072",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "HackerNews"
    },
    {
      "pk": "6b0d4c8d9dc59e110d380b0429a02891f1341a0fa2ba1b1cf83a3db4d47e3964",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "dergigi"
    },
    {
      "pk": "3235036bd0957dfb27ccda02d452d7c763be40c91a1ac082ba6983b25238388c",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "vishalxl"
    },
    {
      "pk": "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "jb55.com"
    }
   ],
  "subscribed_authors": [
    "6b0d4c8d9dc59e110d380b0429a02891f1341a0fa2ba1b1cf83a3db4d47e3964",
    "3235036bd0957dfb27ccda02d452d7c763be40c91a1ac082ba6983b25238388c"
  ],
  "subscribed_pubkeys": [
    "887645fef0ce0c3c1218d2f5d8e6132a19304cdc57cd20281d082f38cfea0072",
    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
  ]
}
```

# Example Usage

```
$ nostr-commander-rs --create-user --name "James Jones" \
    --display-name Jimmy --about "tech and pizza lover" \
    --picture "https://i.imgur.com/mIcObyL.jpeg" \
    --nip05 jim@nostr.example.org \
    --add-relay "wss://nostr.openchain.fr" "wss://relay.damus.io" # first time only
$ nostr-commander-rs --add-contact --key "887645fef0ce0c3c1218d2f5d8e6132a19304cdc57cd20281d082f38cfea0072" --alias HackerNews --relay "wss://nostr.openchain.fr/"
$ nostr-commander-rs --publish "Love this protocol"
$ nostr-commander-rs --dm joe "How about pizza tonight?"
$ nostr-commander-rs --subscribe-author npub1xtscya34g58tk0z605fvr788k263gsu6cy9x0mhnm87echrgufzsevkk5s
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
      --usage
          Prints very short help summary. Details:: See also --help, --manual
          and --readme
  -h, --help
          Prints short help. Details:: See also --usage, --manual and --readme
      --manual
          Prints long help. Details:: See also --usage, --help and --readme
      --readme
          Prints README.md file, the documenation in Markdown. Details:: See
          also --usage, --help and --manual
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
          Publish one or multiple notes. Notes data must not be binary data, it
          must be text. Input piped via stdin can additionally be specified
          with the special character '-'. If you want to feed a text message
          into the program via a pipe, via stdin, then specify the special
          character '-'. If your message is literally a single letter '-' then
          use an escaped '\-' or a quoted "\-". Depending on your shell, '-'
          might need to be escaped. If this is the case for your shell, use the
          escaped '\-' instead of '-' and '\\-' instead of '\-'. However,
          depending on which shell you are using and if you are quoting with
          double quotes or with single quotes, you may have to add backslashes
          to achieve the proper escape sequences. If you want to read the
          message from the keyboard use '-' and do not pipe anything into
          stdin, then a message will be requested and read from the keyboard.
          Keyboard input is limited to one line. The stdin indicator '-' may
          appear in any position, i.e. --publish 'start' '-' 'end' will send 3
          messages out of which the second one is read from stdin. The stdin
          indicator '-' may appear only once overall in all arguments. '-'
          reads everything that is in the pipe in one swoop and sends a single
          message. Similar to '-', another shortcut character is '_'. The
          special character '_' is used for streaming data via a pipe on stdin.
          With '_' the stdin pipe is read line-by-line and each line is treated
          as a separate message and sent right away. The program waits for pipe
          input until the pipe is closed. E.g. Imagine a tool that generates
          output sporadically 24x7. It can be piped, i.e. streamed, into
          nostr-commander, and nostr-commander stays active, sending all input
          instantly. If you want to send the literal letter '_' then escape it
          and send '\_'. '_' can be used only once. And either '-' or '_' can
          be used
      --publish-pow [<NOTE>...]
          Publish one or multiple notes with proof-of-work (POW). Use also
          '--pow-difficulty' to specify difficulty. See also '--publish' to see
          how shortcut characters '-' (pipe) and '_' (streamed pipe) are
          handled
      --dm [<KEY+MSGS>...]
          Send one or multiple DMs to one given user. DM messages will be
          encrypted and preserve privacy. The single recipient is specified via
          its public key, a string in the form of 'npub1...', a Hex key, or an
          alias from one of your contacts. The first argument is the recipient,
          all further arguments are texts to be sent. E.g. '-dm
          "npub1SomeStrangeNumbers" "First msg" "Second msg"' or '--dm joe "How
          about pizza tonight?"'. See also '--publish' to see how shortcut
          characters '-' (pipe) and '_' (streamed pipe) are handled
      --send-channel-message [<HASH+MSGS>...]
          Send one or multiple messages to one given channel. The single
          destination channel is specified via its hash. See here for a channel
          list: https://damus.io/channels/. The first argument is the channel
          hash, all further arguments are texts to be sent. E.g.
          '-send_channel_message "SomeChannelHash" "First msg" "Second msg"'.
          See also '--publish' to see how shortcut characters '-' (pipe) and
          '_' (streamed pipe) are handled
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
      --npub-to-hex [<KEY>...]
          Convert one or multiple public keys in Bech32 format ('npub1...')
          into the corresponding 'hex' format. Details:: See also --hex-to-npub
      --hex-to-npub [<KEY>...]
          Convert one or multiple public keys in 'hex' format into the
          corresponding Bech32 ('npub1...') format. Details:: See also
          --npub-to-hex
      --get-pubkey-entity [<KEY>...]
          Get the entity of one or multiple public keys. Details:: This will
          show you for every public key given if the key represents a Nostr
          account (usually an individual) or a public Nostr channel. It might
          also return "Unknown" if the entity of the key cannot be determined.
          E.g. this can be helpful to determine if you want to use
          --subscribe-author or --subscribe-channel
      --subscribe-pubkey [<KEY>...]
          Subscribe to one or more public keys. Details: Specify each public
          key in form of 'npub1SomePublicKey'. Alternatively you can use the
          Hex form of the public key. Use this option to subscribe to an
          account, i.e. the key of an individual. See also --subscribe-channel
          which are different
      --subscribe-author [<KEY>...]
          Subscribe to authors with to one or more public keys of accounts.
          Details:: Specify each public key in form of 'npub1SomePublicKey'.
          Alternatively you can use the Hex form of the public key. Use this
          option to subscribe to a Nostr accounts (usually individuals).
          Provide keys that represent accounts (see --get-pubkey-entity). See
          also --subscribe-pubkey and --subscribe-channel which are different
      --subscribe-channel [<KEY>...]
          Subscribe to public channels with one or more public keys of
          channels. Details:: Specify each public key in form of
          'npub1SomePublicKey'. Alternatively you can use the Hex form of the
          public key. Sometimes the public key of a public channel is referred
          to as channel id. Provide keys that represent public channels (see
          --get-pubkey-entity). See also --subscribe-pubkey and
          --subscribe-author which are different
      --limit-number <NUMBER>
          Limit the number of messages to receive when subscribing. By default
          there is no limit (0) [default: 0]
      --limit-days <DAYS>
          Limit the messages received to the last N days when subscribing. By
          default there is no limit (0) [default: 0]
      --limit-hours <HOURS>
          Limit the messages received to the last N hours when subscribing. By
          default there is no limit (0) [default: 0]
      --limit-future-days <DAYS>
          Limit the messages received to the next N days when subscribing. Stop
          receiving N days in the future. By default there is no limit (0) [default:
          0]
      --limit-future-hours <HOURS>
          Limit the messages received to the last N hours when subscribing.
          Stop receiving N hours in the future. By default there is no limit
          (0) [default: 0]

```

# Other Related Projects

- Look here for an [nostr awesome list](https://github.com/aljazceru/awesome-nostr).
- `nostr-commander` isn't quite what you wanted?
  Check out [nostr_console](https://github.com/vishalxl/nostr_console).
- Not into `nostr` but into Matrix?
  Check out [matrix-commander](https://github.com/8go/matrix-commander)
  and [matrix-commander-rs](https://github.com/8go/matrix-commander-rs).
- Also [matrix-nostr-bridge](matrix-nostr-bridge).
