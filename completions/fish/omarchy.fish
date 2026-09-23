function __omarchy_commands
  omarchy commands --json 2>/dev/null | jq -r '.commands[] | .group + " " + .name + "\t" + .summary' 2>/dev/null
end

function __omarchy_groups
  omarchy commands --json 2>/dev/null | jq -r '.commands[] | [.group, .summary] | @tsv' 2>/dev/null \
    | sort -u -k1,1
end

function __omarchy_subcommands
  set -l group (commandline -opc)[2]
  omarchy commands --json 2>/dev/null \
    | jq -r --arg g "$group" \
        '.commands[] | select(.group == $g and .name != "") | .name + "\t" + .summary' 2>/dev/null
end

function __omarchy_no_subcommand
  not __fish_seen_subcommand_from (omarchy commands --json 2>/dev/null | jq -r '.commands[].group' 2>/dev/null)
end

# Complete top-level groups
complete -c omarchy -f -n '__omarchy_no_subcommand' \
  -a '(__omarchy_groups)'

# Complete subcommands once a group is chosen
complete -c omarchy -f -n 'not __omarchy_no_subcommand' \
  -a '(__omarchy_subcommands)'
