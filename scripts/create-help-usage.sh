#!/usr/bin/env bash
PATH=".:./target/debug/nostr-commander-rs/:$PATH" &&
    nostr-commander-rs --usage >help.usage.txt
echo "help.usage.txt is $(wc -l help.usage.txt | cut -d ' ' -f1) lines long"
