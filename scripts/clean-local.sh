#!/usr/bin/env bash
# echo "doing small cleanup of debug build"
ls -lh target/debug/incremental/nostr_commander_rs-* target/debug/nostr-commander-rs* 2> /dev/null
rm -r -f target/debug/incremental/nostr_commander_rs-* target/debug/nostr-commander-rs*
