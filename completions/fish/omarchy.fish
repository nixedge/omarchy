function __omarchy_groups
    omarchy commands --json 2>/dev/null \
        | jq -r '[.commands[].group] | unique | .[]' 2>/dev/null
end

function __omarchy_subcommands
    set -l group (commandline -opc)[2]
    omarchy commands --json 2>/dev/null \
        | jq -r --arg g "$group" \
            '.commands[] | select(.group == $g and .name != "") | .name + "\t" + .summary' \
            2>/dev/null
end

function __omarchy_needs_group
    test (count (commandline -opc)) -lt 2
end

complete -c omarchy -f -n '__omarchy_needs_group' -a '(__omarchy_groups)'
complete -c omarchy -f -n 'not __omarchy_needs_group' -a '(__omarchy_subcommands)'
