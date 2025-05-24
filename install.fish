#!/usr/bin/env fish

# Script to install rkit shell functions
# This script adds useful functions to your fish configuration file
# for working with rkit repositories.

# Check if rkit is installed
if not command -v rkit > /dev/null
    echo "Error: rkit is not installed"
    echo "Please install rkit first by running:"
    echo "cargo install rkit"
    exit 1
end

# Check if fzf is installed
if not command -v fzf > /dev/null
    echo "Error: fzf is not installed"
    echo "Please install fzf first by following the instructions at:"
    echo "https://junegunn.github.io/fzf/"
    exit 1
end

# Function to add to fish config
function add_to_fish_config
    set config_file "$HOME/.config/fish/config.fish"

    # Create backup of config file
    if test -f "$config_file"
        cp "$config_file" "$config_file.bak"
        echo "Created backup of $config_file at $config_file.bak"
    end

    # Initialize the functions string
    set functions_to_add ""

    # Check and ask for clone function
    if not grep -q "^function clone" "$config_file"
        echo "The following function will be added to $config_file:"
        echo
        echo "function clone"
        echo "    rkit clone \$argv"
        echo "end"
        echo
        read -l -P "Do you want to install the clone function? (y/N) " reply
        if test "$reply" = "y" -o "$reply" = "Y"
            set functions_to_add "$functions_to_add

function clone
    rkit clone \$argv
end"
        end
    else
        echo "clone function is already installed in $config_file"
    end

    # Check and ask for cdc function
    if not grep -q "^function cdc" "$config_file"
        echo "The following function will be added to $config_file:"
        echo
        echo "function cdc"
        echo "    set -l query \$argv[1]"
        echo "    if test -n \"\$query\""
        echo "        cd (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")"
        echo "    else"
        echo "        cd (rkit ls | fzf --preview 'rkit view {}')"
        echo "    end"
        echo "end"
        echo
        read -l -P "Do you want to install the cdc function? (y/N) " reply
        if test "$reply" = "y" -o "$reply" = "Y"
            set functions_to_add "$functions_to_add

function cdc
    set -l query \$argv[1]
    if test -n \"\$query\"
        cd (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")
    else
        cd (rkit ls | fzf --preview 'rkit view {}')
    end
end"
        end
    else
        echo "cdc function is already installed in $config_file"
    end

    # Check and ask for edit function
    if not grep -q "^function edit" "$config_file"
        echo "The following function will be added to $config_file:"
        echo
        echo "function edit"
        echo "    set -l query \$argv[1]"
        echo "    if test -n \"\$query\""
        echo "        code (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")"
        echo "    else"
        echo "        code (rkit ls | fzf --preview 'rkit view {}')"
        echo "    end"
        echo "end"
        echo
        read -l -P "Do you want to install the edit function? (y/N) " reply
        if test "$reply" = "y" -o "$reply" = "Y"
            set functions_to_add "$functions_to_add

function edit
    set -l query \$argv[1]
    if test -n \"\$query\"
        code (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")
    else
        code (rkit ls | fzf --preview 'rkit view {}')
    end
end"
        end
    else
        echo "edit function is already installed in $config_file"
    end

    # Add completion configuration
    if not grep -q "complete -c edit" "$config_file"
        echo "The following completion configuration will be added to $config_file:"
        echo
        echo "# rkit completions"
        echo "complete -c edit -a '(rkit ls)'"
        echo "complete -c cdc -a '(rkit ls)'"
        echo
        read -l -P "Do you want to install the completion configuration? (y/N) " reply
        if test "$reply" = "y" -o "$reply" = "Y"
            set functions_to_add "$functions_to_add

# rkit completions
complete -c edit -a '(rkit ls)'
complete -c cdc -a '(rkit ls)'"
        end
    else
        echo "Completion configuration is already installed in $config_file"
    end

    # If no functions were selected, exit
    if test -z "$functions_to_add"
        echo "No new functions were selected for installation."
        return
    end

    # Add the selected functions to the config file
    echo "# rkit functions$functions_to_add" >> "$config_file"

    echo "Added selected functions to $config_file"
    echo "Please restart your shell or run 'source $config_file' to use the new functions"
end

# Main installation
echo "Installing rkit functions for fish..."
add_to_fish_config 