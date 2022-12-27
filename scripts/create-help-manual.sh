#!/usr/bin/env bash
PATH=".:./target/debug/nostr-commander-rs/:$PATH" &&
    nostr-commander-rs --manual >help.manual.txt
echo "help.manual.txt is $(wc -l help.manual.txt | cut -d ' ' -f1) lines long"
