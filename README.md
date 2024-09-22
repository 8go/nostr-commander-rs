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
Thanks to all that have already contributed. Shout out to [@ntheden](https://github.com/ntheden).  

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
    {
      "url": "wss://nostr-pub.wellorder.net/",
      "proxy": null
    }
  ],
  "metadata": {
    "name": "James Jones",
    "display_name": "Jim",
    "about": "tech nerd and nostr lover"
  },
  "contacts": [
    {
      "pk": "25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "Nostr Public Channel"
    },
    {
      "pk": "887645fef0ce0c3c1218d2f5d8e6132a19304cdc57cd20281d082f38cfea0072",
      "relay_url": "wss://nostr-pub.wellorder.net/",
      "alias": "HackerNews Public Channel"
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
  "subscribed_pubkeys": [
    "3235036bd0957dfb27ccda02d452d7c763be40c91a1ac082ba6983b25238388c"
  ],
  "subscribed_authors": [
    "6b0d4c8d9dc59e110d380b0429a02891f1341a0fa2ba1b1cf83a3db4d47e3964",
    "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
  ],
  "subscribed_channels": [
    "25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb",
    "887645fef0ce0c3c1218d2f5d8e6132a19304cdc57cd20281d082f38cfea0072"
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
$ nostr-commander-rs --subscribe-channel 25e5c82273a271cb1a840d0060391a0bf4965cafeb029d5ab55350b418953fbb
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
          Details:: If used without an argument such as '--version' it will
          print the version number. If 'check' is added ('--version check')
          then the program connects to https://crates.io and gets the version
          number of latest stable release. There is no "calling home" on every
          run, only a "check crates.io" upon request. Your privacy is
          protected. New release is neither downloaded, nor installed. It just
          informs you

          Possible values:
          - check: Check if there is a newer version available

      --usage
          Prints a very short help summary. Details:: See also --help, --manual
          and --readme

  -h, --help
          Prints short help displaying about one line per argument. Details::
          See also --usage, --manual and --readme

      --manual
          Prints long help. Details:: This is like a man page. See also
          --usage, --help and --readme

      --readme
          Prints README.md file, the documenation in Markdown. Details:: The
          README.md file will be downloaded from Github. It is a Markdown file
          and it is best viewed with a Markdown viewer. See also --usage,
          --help and --manual

  -d, --debug...
          Overwrite the default log level. Details:: If not used, then the
          default log level set with environment variable 'RUST_LOG' will be
          used. If used, log level will be set to 'DEBUG' and debugging
          information will be printed. '-d' is a shortcut for '--log-level
          DEBUG'. See also '--log-level'. '-d' takes precedence over
          '--log-level'. Additionally, have a look also at the option
          '--verbose'

      --log-level <LOG_LEVEL>
          Set the log level by overwriting the default log level. Details:: If
          not used, then the default log level set with environment variable
          'RUST_LOG' will be used. See also '--debug' and '--verbose'
          
          [default: none]

          Possible values:
          - none:  None: not set, default
          - error: Error: Indicates to print only errors
          - warn:  Warn: Indicates to print warnings and errors
          - info:  Info: Indicates to to print info, warn and errors
          - debug: Debug: Indicates to to print debug and the rest
          - trace: Trace: Indicates to to print everything

      --verbose...
          Set the verbosity level. Details:: If not used, then verbosity will
          be set to low. If used once, verbosity will be high. If used more
          than once, verbosity will be very high. Verbosity only affects the
          debug information. So, if '--debug' is not used then '--verbose' will
          be ignored

  -c, --credentials <PATH_TO_FILE>
          Specify a path to a file containing credentials. Details:: At
          --create-user, information about the user, in particular its keys,
          will be written to a credentials file. By default, this file is
          "credentials.json". On further runs the credentials file is read to
          permit acting as this established Nostr user. If this option is
          provided, the provided path to a file will be used as credentials
          file instead of the default one
          
          [default:
          /home/user/.local/share/nostr-commander-rs/credentials.json]

      --create-user
          Create a new user, i.e. a new key pair. Details:: This is usually
          done only once at the beginning. If you ever want to wipe this user,
          use '--delete-user' which deletes the key pair. Use this option in
          combination with --name, --display_name, --about, --picture, and
          --nip05. Also highly recommended that you use this option together
          with --add-relay

      --delete-user
          Delete the current user, i.e. delete the current key pair. Details::
          This will erase the key pair and other associated information like
          user name, display name, etc. Afterwards one can create a new user
          with '--create-user'

      --name <USER_NAME>
          Specify an optional user name. Details:: Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''

      --display-name <DISPLAY_NAME>
          Specify an optional display name. Details:: Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''

      --about <DESCRIPTION>
          Specify an optional description. Details:: Used together with
          '--create-user'. If this option is not set during '--create-user',
          the information will be queried via the keyboard. If you want to set
          it to empty and not be queried, provide an empty string ''

      --picture <URL>
          Specify an optional picture or avatar. Details:: Used together with
          '--create-user'. Provide a URL like 'https://example.com/avatar.png'.
          If this option is not set during '--create-user', the information
          will be queried via the keyboard. If you want to set it to empty and
          not be queried, provide this URL 'none:'

      --nip05 <NIP05_ID>
          Specify an optional nip05 name. Details:: Used together with
          '--create-user'. Provide a nip05 name like 'john@example.org'. If
          this option is not set during '--create-user', the information will
          be queried via the keyboard. If you want to set it to empty and not
          be queried, provide an empty string ''

  -p, --publish [<NOTE>...]
          Publish one or multiple notes. Details:: Notes data must not be
          binary data, it must be text. Input piped via stdin can additionally
          be specified with the special character '-'. If you want to feed a
          text message into the program via a pipe, via stdin, then specify the
          special character '-'. If your message is literally a single letter
          '-' then use an escaped '\-' or a quoted "\-". Depending on your
          shell, '-' might need to be escaped. If this is the case for your
          shell, use the escaped '\-' instead of '-' and '\\-' instead of '\-'.
          However, depending on which shell you are using and if you are
          quoting with double quotes or with single quotes, you may have to add
          backslashes to achieve the proper escape sequences. If you want to
          read the message from the keyboard use '-' and do not pipe anything
          into stdin, then a message will be requested and read from the
          keyboard. Keyboard input is limited to one line. The stdin indicator
          '-' may appear in any position, i.e. --publish 'start' '-' 'end' will
          send 3 messages out of which the second one is read from stdin. The
          stdin indicator '-' may appear only once overall in all arguments.
          '-' reads everything that is in the pipe in one swoop and sends a
          single message. Similar to '-', another shortcut character is '_'.
          The special character '_' is used for streaming data via a pipe on
          stdin. With '_' the stdin pipe is read line-by-line and each line is
          treated as a separate message and sent right away. The program waits
          for pipe input until the pipe is closed. E.g. Imagine a tool that
          generates output sporadically 24x7. It can be piped, i.e. streamed,
          into nostr-commander, and nostr-commander stays active, sending all
          input instantly. If you want to send the literal letter '_' then
          escape it and send '\_'. '_' can be used only once. And either '-' or
          '_' can be used

      --publish-pow [<NOTE>...]
          Publish one or multiple notes with proof-of-work (POW). Details:: Use
          also '--pow-difficulty' to specify difficulty. See also '--publish'
          to see how shortcut characters '-' (pipe) and '_' (streamed pipe) are
          handled. Disabled since version nostr-commander-rs 0.2.0 (nostr-sdk
          0.21)

      --dm [<KEY+MSGS>...]
          Send one or multiple DMs to one given user. Details:: DM messages
          will be encrypted and preserve privacy. The single recipient is
          specified via its public key, a string in the form of 'npub1...', a
          Hex key, or an alias from one of your contacts. The first argument is
          the recipient, all further arguments are texts to be sent. E.g. '-dm
          "npub1SomeStrangeNumbers" "First msg" "Second msg"' or '--dm joe "How
          about pizza tonight?"'. See also '--publish' to see how shortcut
          characters '-' (pipe) and '_' (streamed pipe) are handled

      --send-channel-message [<HASH+MSGS>...]
          Send one or multiple messages to one given channel. Details:: The
          single destination channel is specified via its hash. See here for a
          channel list: https://damus.io/channels/. The first argument is the
          channel hash, all further arguments are texts to be sent. E.g.
          '-send_channel_message "SomeChannelHash" "First msg" "Second msg"'.
          See also '--publish' to see how shortcut characters '-' (pipe) and
          '_' (streamed pipe) are handled. Optionally you can provide a relay
          to be used for the channel send by using --relay. See --relay. If
          --relay has values the first value from --relay will be used as
          relay. If --relay is not used, then the first relay in the relay list
          in the credentials configuration file will be used

      --add-relay [<RELAY_URI>...]
          Add one or multiple relays. Details:: A relay is specified via a URI
          that looks like 'wss://some.relay.org'. You can find relays by
          looking at https://github.com/aljazceru/awesome-nostr#instances.
          Sampler relay registries are: https://nostr-registry.netlify.app/,
          https://nostr.info/, or https://nostr.watch/. Examples:
          "wss://relay.damus.io", "wss://nostr.openchain.fr". See also
          '--proxy'

      --proxy <PROXY>
          Specify a proxy for relays. Details:: Used by --add-relay. Note that
          this proxy will be applied to all of the relays specified with
          --add-relay. If you have 3 relays with 3 different proxies, then run
          the --add-relay command 3 times with 1 relay and 1 proxy each time.
          An example proxy for the Tor network looks something like
          "127.0.0.1:9050". If you want to use Tor via a proxy, to assure that
          no information leaks you must use only one relay, i.e. the Tor relay.
          If more then one relays are configured, data will be communicated to
          and from all relays. A possible relay that you can use together with
          a Tor proxy is
          "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion"

      --remove-relay [<RELAY_URI>...]
          Remove one or multiple relays from local config file. Details:: See
          --add-relay

      --tag <TAG>
          Specify one or multiple tags to attach to notes or DMs. Details:: Not
          yet implemented

      --show-metadata
          Display current metadata. Details:: shows data in your config file

      --change-metadata
          Modify existing metadata of the user. Details:: Use this option in
          combination with --name, --display_name, --about, --picture, and
          --nip05

      --pow-difficulty <DIFFICULTY>
          Specify optional proof-of-work (POW) difficulty. Details:: Use with
          '--publish_pow' to specify difficulty. If not specified the default
          will be used
          
          [default: 20]

      --show-public-key
          Show public key. Details:: Displays your own public key. You can
          share this with your friends or the general public

      --show-secret-key
          Show private, secret key. Details:: Protect this key. Do not share
          this with anyone

      --whoami
          Print the user name used by "nostr-commander-rs". Details:: One can
          get this information also by looking at the credentials file or by
          using --show-metadata

  -o, --output <OUTPUT_FORMAT>
          Select an output format. Details:: This option decides on how the
          output is presented. Currently offered choices are: 'text', 'json',
          'json-max', and 'json-spec'. Provide one of these choices. The
          default is 'text'. If you want to use the default, then there is no
          need to use this option. If you have chosen 'text', the output will
          be formatted with the intention to be consumed by humans, i.e.
          readable text. If you have chosen 'json', the output will be
          formatted as JSON. The content of the JSON object matches the data
          provided by the nostr-sdk SDK. In some occassions the output is
          enhanced by having a few extra data items added for convenience. In
          most cases the output will be processed by other programs rather than
          read by humans. Option 'json-max' is practically the same as 'json',
          but yet another additional field is added. In most cases the output
          will be processed by other programs rather than read by humans.
          Option 'json-spec' only prints information that adheres 1-to-1 to the
          Nostr Specification. Currently this type is not supported. If no data
          is available that corresponds exactly with the Nostr Specification,
          no data will be printed
          
          [default: text]

          Possible values:
          - text:      Text: Indicates to print human readable text, default
          - json:      Json: Indicates to print output in Json format
          - json-max:  Json Max: Indicates to to print the maximum anount of
            output in Json format
          - json-spec: Json Spec: Indicates to to print output in Json format,
            but only data that is according to Nostr Specifications

  -l, --listen
          Listen to events, notifications and messages. Details:: This option
          listens to events and messages forever. To stop, type Control-C on
          your keyboard. You want to listen if you want to get the event ids
          for published notices. Subscriptions do not automatically turn
          listening on. If you want to listen to your subscriptions, you must
          use --listen

      --add-contact
          Add one or more contacts. Details:: Must be used in combination with
          --alias, --key, --relay. If you want to add N new contacts, use
          --add-contact and provide exactly N entries in each of the 3 extra
          arguments. E.g. --add-contact --alias jane joe --key
          npub1JanesPublicKey npub1JoesPublicKey --relay
          "wss://janes.relay.org" "wss://joes.relay.org". Aliases must be
          unique. Alias can be seen as a nickname

      --remove-contact
          Remove one or more contacts. Details:: Must be used in combination
          with --alias. For each entry in --alias the corresponding contact
          will be removed. E.g. --remove-contact --alias jane joe

      --show-contacts
          Display current contacts. Details:: Prints your contact list

      --alias [<ALIAS>...]
          Provide one or multiple aliases (nicknames). Details:: This is used
          in combination with arguments --add-contact and --remove-contact

      --key [<KEY>...]
          Provide one or multiple public keys. Details:: This is used in
          combination with argument --add-contact. They have the form
          'npub1SomeStrangeString'. Alternatively you can use the Hex form of
          the public key

      --relay [<RELAY>...]
          Provide one or multiple relays. Details:: This is used in combination
          with arguments --add-contact and --send_channel_message. Relays have
          the form 'wss://some.relay.org'

      --npub-to-hex [<KEY>...]
          Convert one or multiple public keys from Npub to Hex. Details::
          Converts public keys in Bech32 format ('npub1...') into the
          corresponding 'hex' format. See also --hex-to-npub

      --hex-to-npub [<KEY>...]
          Convert one or multiple public keys from Hex to Npub. Details::
          Converts public keys in 'hex' format into the corresponding Bech32
          ('npub1...') format. See also --npub-to-hex

      --get-pubkey-entity [<KEY>...]
          Get the entity of one or multiple public keys. Details:: This will
          show you for every public key given if the key represents a Nostr
          account (usually an individual) or a public Nostr channel. It might
          also return "Unknown" if the entity of the key cannot be determined.
          E.g. this can be helpful to determine if you want to use
          --subscribe-author or --subscribe-channel

      --subscribe-pubkey [<KEY>...]
          Subscribe to one or more public keys. Details:: Specify each public
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

      --subscribe-channel [<HASH>...]
          Subscribe to public channels with one or more hashes of channels.
          Details:: Specify each hash in form of 'npub1SomePublicKey'.
          Alternatively you can use the Hex form of the public key. Sometimes
          the hash of a public channel is referred to as channel id, sometimes
          also as public channel key. See here for a channel list:
          https://damus.io/channels/. Provide hashes that represent public
          channels (see --get-pubkey-entity). See also --subscribe-pubkey and
          --subscribe-author which are different

      --unsubscribe-pubkey [<KEY>...]
          Unsubscribe from public key. Details:: Removes one or multiple public
          keys from the public key subscription list. See --subscribe-pubkey

      --unsubscribe-author [<KEY>...]
          Unsubscribe from author. Details:: Removes one or multiple public
          keys from the author subscription list. See --subscribe-author

      --unsubscribe-channel [<KEY>...]
          Unsubscribe from public channel. Details:: Removes one or multiple
          public keys from the public channel subscription list. See
          --subscribe-channel

      --limit-number <NUMBER>
          Limit the number of past messages to receive when subscribing.
          Details:: By default there is no limit (0), i.e. all old messages
          available to the relay will be received
          
          [default: 0]

      --limit-days <DAYS>
          Limit the messages received to the last N days when subscribing.
          Details:: By default there is no limit (0), i.e. all old messages
          available to the relay will be received
          
          [default: 0]

      --limit-hours <HOURS>
          Limit the messages received to the last N hours when subscribing.
          Details:: By default there is no limit (0), i.e. all old messages
          available to the relay will be received
          
          [default: 0]

      --limit-future-days <DAYS>
          Limit the messages received to the next N days when subscribing.
          Details:: Stop receiving N days in the future. By default there is no
          limit (0), i.e. you will receive events forever
          
          [default: 0]

      --limit-future-hours <HOURS>
          Limit the messages received to the last N hours when subscribing.
          Details:: Stop receiving N hours in the future. By default there is
          no limit (0), i.e. you will receive events forever
          
          [default: 0]

PS: Also have a look at scripts/nostr-commander-tui.

```

# Other Related Projects

- Look here for an [nostr awesome list](https://github.com/aljazceru/awesome-nostr).
- `nostr-commander` isn't quite what you wanted?
  Check out [nostr_console](https://github.com/vishalxl/nostr_console).
- Not into `nostr` but into Matrix?
  Check out [matrix-commander](https://github.com/8go/matrix-commander)
  and [matrix-commander-rs](https://github.com/8go/matrix-commander-rs).
- Also [matrix-nostr-bridge](matrix-nostr-bridge).
