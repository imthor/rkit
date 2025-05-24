#!/usr/bin/env bash

# Exit on error, undefined variables, and pipeline failures
set -euo pipefail

# Script to install rkit shell functions
# This script adds useful functions to your shell configuration file
# for working with rkit repositories.

# Check if rkit is installed
if ! command -v rkit &> /dev/null; then
    echo "Error: rkit is not installed"
    echo "Please install rkit first by running:"
    echo "cargo install rkit"
    exit 1
fi

# Check if fzf is installed
if ! command -v fzf &> /dev/null; then
    echo "Error: fzf is not installed"
    echo "Please install fzf first by following the instructions at:"
    echo "https://junegunn.github.io/fzf/"
    exit 1
fi

# Detect the shell
SHELL_NAME=$(basename "$SHELL")

# Function to add to shell config
# Parameters:
#   None
# Returns:
#   None
add_to_shell_config() {
    local config_file
    if [ "$SHELL_NAME" = "zsh" ]; then
        config_file="$HOME/.zshrc"
    else
        config_file="$HOME/.bashrc"
    fi

    # Create backup of config file
    if [ -f "$config_file" ]; then
        cp "$config_file" "${config_file}.bak"
        echo "Created backup of $config_file at ${config_file}.bak"
    fi

    # Initialize the functions string
    local functions_to_add=""

    # Check and ask for clone function
    if grep -q "^clone()" "$config_file"; then
        echo "clone function is already installed in $config_file"
    else
        echo "The following function will be added to $config_file:"
        echo
        echo "clone() {"
        echo "    rkit clone \"\$@\""
        echo "}"
        echo
        read -p "Do you want to install the clone function? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            functions_to_add+="
clone() {
    rkit clone \"\$@\"
}"
        fi
    fi

    # Check and ask for cdc function
    if grep -q "^cdc()" "$config_file"; then
        echo "cdc function is already installed in $config_file"
    else
        echo "The following function will be added to $config_file:"
        echo
        echo "cdc() {"
        echo "    local query=\"\$1\""
        echo "    if [ -n \"\$query\" ]; then"
        echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
        echo "    else"
        echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
        echo "    fi"
        echo "}"
        echo
        read -p "Do you want to install the cdc function? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            functions_to_add+="
cdc() {
    local query=\"\$1\"
    if [ -n \"\$query\" ]; then
        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\"
    else
        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}')\"
    fi
}"
        fi
    fi

    # Check and ask for edit function
    if grep -q "^edit()" "$config_file"; then
        echo "edit function is already installed in $config_file"
    else
        echo "The following function will be added to $config_file:"
        echo
        echo "edit() {"
        echo "    local query=\"\$1\""
        echo "    if [ -n \"\$query\" ]; then"
        echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
        echo "    else"
        echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
        echo "    fi"
        echo "}"
        echo
        read -p "Do you want to install the edit function? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            functions_to_add+="
edit() {
    local query=\"\$1\"
    if [ -n \"\$query\" ]; then
        code \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\"
    else
        code \"\$(rkit ls -f | fzf --preview 'rkit view {}')\"
    fi
}"
        fi
    fi

    # Add autocomplete configuration
    if [ "$SHELL_NAME" = "zsh" ]; then
        if ! grep -q "_rkit_completion" "$config_file"; then
            echo "The following zsh completion configuration will be added to $config_file:"
            echo
            echo "# rkit completion for zsh"
            echo "autoload -Uz compinit"
            echo "compinit"
            echo
            echo "_rkit_completion() {"
            echo "    local curcontext=\"\$curcontext\" state line"
            echo "    typeset -A opt_args"
            echo
            echo "    _arguments -C \\"
            echo "        '1: :->cmds' \\"
            echo "        '*::arg:->args'"
            echo
            echo "    case \$state in"
            echo "        cmds)"
            echo "            _values 'rkit commands' \\"
            echo "                \$(rkit ls)"
            echo "            ;;"
            echo "        args)"
            echo "            case \$line[1] in"
            echo "                edit|cdc)"
            echo "                    _values 'repositories' \\"
            echo "                        \$(rkit ls)"
            echo "                    ;;"
            echo "            esac"
            echo "            ;;"
            echo "    esac"
            echo "}"
            echo
            echo "compdef _rkit_completion edit cdc"
            echo
            read -p "Do you want to install the zsh completion configuration? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                functions_to_add+="
# rkit completion for zsh
autoload -Uz compinit
compinit

_rkit_completion() {
    local curcontext=\"\$curcontext\" state line
    typeset -A opt_args

    _arguments -C \\
        '1: :->cmds' \\
        '*::arg:->args'

    case \$state in
        cmds)
            _values 'rkit commands' \\
                \$(rkit ls)
            ;;
        args)
            case \$line[1] in
                edit|cdc)
                    _values 'repositories' \\
                        \$(rkit ls)
                    ;;
            esac
            ;;
    esac
}

compdef _rkit_completion edit cdc"
            fi
        fi
    else
        if ! grep -q "_rkit_completion" "$config_file"; then
            echo "The following bash completion configuration will be added to $config_file:"
            echo
            echo "# rkit completion for bash"
            echo "_rkit_completion() {"
            echo "    local cur prev opts"
            echo "    COMPREPLY=()"
            echo "    cur=\"\${COMP_WORDS[COMP_CWORD]}\""
            echo "    prev=\"\${COMP_WORDS[COMP_CWORD-1]}\""
            echo
            echo "    case \${prev} in"
            echo "        edit|cdc)"
            echo "            COMPREPLY=( \$(compgen -W \"\$(rkit ls)\" -- \${cur}) )"
            echo "            return 0"
            echo "            ;;"
            echo "    esac"
            echo "}"
            echo
            echo "complete -F _rkit_completion edit cdc"
            echo
            read -p "Do you want to install the bash completion configuration? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                functions_to_add+="
# rkit completion for bash
_rkit_completion() {
    local cur prev opts
    COMPREPLY=()
    cur=\"\${COMP_WORDS[COMP_CWORD]}\"
    prev=\"\${COMP_WORDS[COMP_CWORD-1]}\"

    case \${prev} in
        edit|cdc)
            COMPREPLY=( \$(compgen -W \"\$(rkit ls)\" -- \${cur}) )
            return 0
            ;;
    esac
}

complete -F _rkit_completion edit cdc"
            fi
        fi
    fi

    # If no functions were selected, exit
    if [ -z "$functions_to_add" ]; then
        echo "No new functions were selected for installation."
        return
    fi

    # Add the selected functions to the config file
    echo "# rkit functions$functions_to_add" >> "$config_file"

    echo "Added selected functions to $config_file"
    echo "Please restart your shell or run 'source $config_file' to use the new functions"
}

# Main installation
echo "Installing rkit functions for $SHELL_NAME..."
add_to_shell_config 