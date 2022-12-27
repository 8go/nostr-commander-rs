#!/usr/bin/env bash

# unused: nostr-commander-rs --help creates a similar output

# Creates a file like this:
# <--usage>
# Print usage.
# <-h>, <--help>
# Print help.
# <--manual>
# Print manual.
# <-d>, <--debug>
# Print debug information.
# <--log-level> DEBUG|INFO|WARNING|ERROR|CRITICAL [DEBUG|INFO|WARNING|ERROR|CRITICAL]
# Set log level.
# <--verbose>
# Set verbosity.
# Always pairs of lines: header line, content line ...

PATH=".:./target/release/:./target/debug/:$PATH"
old_width=$(stty size | cut -d' ' -f2-)
stty cols 1000

# delete everything before (and including) line starting with "Options:"
# delete everything after (and including) line starting with "PS:"
# delete lines starting with "          [default:"
# delete lines starting with "          [default:" up to "]"
# delete lines starting with "          Possible values:"
# delete all lines starting with "             "
# delete first 6 characters of each line
# concatenate all description lines
# delete everything to the right of "Details::"
# remove "   " if at start of line
# replace double newlines with single newlines
nostr-commander-rs --manual  2> /dev/null | 
    sed '1,/^Options:/d' |
    sed '/^PS:/,$d' |
    sed '/^          \[default: /d' | 
    sed '/^          \[default/,/\]/d' |
    sed '/^          Possible values:$/d' |        
    sed '/^          - [a-zA-Z00-9_-]*:.*$/d' |
    sed '/^            /d' |
    sed 's/^......//' |
    sed -e :a -e '$!N;s/\(^ .*\)\n [ ]*/\1 /;ta' -e 'P;D' |
    sed 's/\(.*\)Details::\(.*\)/\1/g' |
    sed 's/^    //' |
    sed ':a;N;$!ba;s/\n\n\n/\n\n/g'      >help.help.pre.txt
    # sed 's/^  //g' | 
    # sed '/^-/ s/  [ ]*/  \n                      /g' |
    # sed -e :a -e '$!N;s/\(^ .*\)\n [ ]*/\1 /;ta' -e 'P;D' |
    # sed 's/^ [ ]*//g' | sed '/^$/d' | sed 's/\(.*\)Details::\(.*\)/\1/g' |
    # sed 's/[ \t]*$//' |
    # sed 's/\(^--[^ ]*\)\(.*\)/<\1>\2/g' |
    # sed 's/\(^-[a-z0-9]\)\(.*\)/<\1>\2/g' |
    # sed 's/\(^<-[^,]*\) \(--[^ ]+\)\(.*\)/\1<\2>\3/g' |
    # sed 's/\(^<-.*\)\(--[^ ]*\)\(.*\)/\1<\2>\3/g' >help.help.pre.txt

stty cols $old_width
stty size
echo -n "Max width: "
wc -L help.help.pre.txt
echo -n "Number of lines: "
wc -l help.help.pre.txt
echo -n "Control Number of options: "
nostr-commander-rs --manual 2> /dev/null | sed 's/^......//' | grep -c '^--'
echo "Control number should be 1/3 of help file line count."
