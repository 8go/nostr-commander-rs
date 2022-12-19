#!/usr/bin/env bash
PATH=".:./target/debug/nostr-commander-rs/:$PATH" &&
    nostr-commander-rs --help >help.help.txt
echo "help.help.txt is $(wc -l help.help.txt | cut -d ' ' -f1) lines long"
