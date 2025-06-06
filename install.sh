#!/usr/bin/env bash

# Exit on error, undefined variables, and pipeline failures
set -euo pipefail

# Script to install rkit shell functions
# This script adds useful functions to your shell configuration file
# for working with rkit repositories.

# Detect the shell and its environment
if [ -n "${ZSH_VERSION:-}" ]; then
    SHELL_NAME="zsh"
    # Get the parent shell's config file
    if [ -f "$HOME/.zshrc" ]; then
        CONFIG_FILE="$HOME/.zshrc"
    else
        echo "Error: Could not find .zshrc in your home directory"
        exit 1
    fi
elif [ -n "${BASH_VERSION:-}" ]; then
    SHELL_NAME="bash"
    if [ -f "$HOME/.bashrc" ]; then
        CONFIG_FILE="$HOME/.bashrc"
    else
        echo "Error: Could not find .bashrc in your home directory"
        exit 1
    fi
else
    echo "Error: Unsupported shell. This script only supports bash and zsh."
    exit 1
fi

# Check if running in interactive mode
if [ -t 0 ]; then
    INTERACTIVE=true
else
    INTERACTIVE=false
    echo "Running in non-interactive mode. All functions will be installed by default."
fi

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

# Function to prompt user for input
prompt_user() {
    local prompt="$1"
    if [ "$INTERACTIVE" = true ]; then
        echo -n "$prompt"
        read -r response
        echo "$response"
    else
        echo "y"  # Default to yes in non-interactive mode
    fi
}

# Function to add to shell config
# Parameters:
#   None
# Returns:
#   None
add_to_shell_config() {
    # Create backup of config file
    if [ -f "$CONFIG_FILE" ]; then
        cp "$CONFIG_FILE" "${CONFIG_FILE}.bak"
        echo "Created backup of $CONFIG_FILE at ${CONFIG_FILE}.bak"
    fi

    # Initialize the functions string
    local functions_to_add=""

    # Check and ask for clone function
    if grep -q "^clone()" "$CONFIG_FILE" || grep -q "^function clone" "$CONFIG_FILE"; then
        echo "clone function is already installed in $CONFIG_FILE"
    else
        echo "The following function will be added to $CONFIG_FILE:"
        echo
        if [ "$SHELL_NAME" = "bash" ]; then
            echo "function clone {"
            echo "    rkit clone \"\$@\""
            echo "}"
        else
            echo "clone() {"
            echo "    rkit clone \"\$@\""
            echo "}"
        fi
        echo
        response=$(prompt_user "Do you want to install the clone function? (y/N) ")
        if [[ $response =~ ^[Yy]$ ]]; then
            if [ "$SHELL_NAME" = "bash" ]; then
                functions_to_add+="
function clone {
    rkit clone \"\$@\"
}"
            else
                functions_to_add+="
clone() {
    rkit clone \"\$@\"
}"
            fi
        fi
    fi

    # Check and ask for cdc function
    if grep -q "^cdc()" "$CONFIG_FILE" || grep -q "^function cdc" "$CONFIG_FILE"; then
        echo "cdc function is already installed in $CONFIG_FILE"
    else
        echo "The following function will be added to $CONFIG_FILE:"
        echo
        if [ "$SHELL_NAME" = "bash" ]; then
            echo "function cdc {"
            echo "    local query=\"\$1\""
            echo "    if [ -n \"\$query\" ]; then"
            echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
            echo "    else"
            echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
            echo "    fi"
            echo "}"
        else
            echo "cdc() {"
            echo "    local query=\"\$1\""
            echo "    if [ -n \"\$query\" ]; then"
            echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
            echo "    else"
            echo "        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
            echo "    fi"
            echo "}"
        fi
        echo
        response=$(prompt_user "Do you want to install the cdc function? (y/N) ")
        if [[ $response =~ ^[Yy]$ ]]; then
            if [ "$SHELL_NAME" = "bash" ]; then
                functions_to_add+="
function cdc {
    local query=\"\$1\"
    if [ -n \"\$query\" ]; then
        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\"
    else
        cd \"\$(rkit ls -f | fzf --preview 'rkit view {}')\"
    fi
}"
            else
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
    fi

    # Check and ask for edit function
    if grep -q "^edit()" "$CONFIG_FILE" || grep -q "^function edit" "$CONFIG_FILE"; then
        echo "edit function is already installed in $CONFIG_FILE"
    else
        echo "The following function will be added to $CONFIG_FILE:"
        echo
        if [ "$SHELL_NAME" = "bash" ]; then
            echo "function edit {"
            echo "    local query=\"\$1\""
            echo "    if [ -n \"\$query\" ]; then"
            echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
            echo "    else"
            echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
            echo "    fi"
            echo "}"
        else
            echo "edit() {"
            echo "    local query=\"\$1\""
            echo "    if [ -n \"\$query\" ]; then"
            echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\""
            echo "    else"
            echo "        code \"\$(rkit ls -f | fzf --preview 'rkit view {}')\""
            echo "    fi"
            echo "}"
        fi
        echo
        response=$(prompt_user "Do you want to install the edit function? (y/N) ")
        if [[ $response =~ ^[Yy]$ ]]; then
            if [ "$SHELL_NAME" = "bash" ]; then
                functions_to_add+="
function edit {
    local query=\"\$1\"
    if [ -n \"\$query\" ]; then
        code \"\$(rkit ls -f | fzf --preview 'rkit view {}' --query \"\$query\")\"
    else
        code \"\$(rkit ls -f | fzf --preview 'rkit view {}')\"
    fi
}"
            else
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
    fi

    # Add autocomplete configuration
    if [ "$SHELL_NAME" = "zsh" ]; then
        if ! grep -q "_rkit_completion" "$CONFIG_FILE"; then
            echo "The following zsh completion configuration will be added to $CONFIG_FILE:"
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
            response=$(prompt_user "Do you want to install the zsh completion configuration? (y/N) ")
            if [[ $response =~ ^[Yy]$ ]]; then
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
        if ! grep -q "_rkit_completion" "$CONFIG_FILE"; then
            echo "The following bash completion configuration will be added to $CONFIG_FILE:"
            echo
            echo "# rkit completion for bash"
            echo "function _rkit_completion {"
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
            response=$(prompt_user "Do you want to install the bash completion configuration? (y/N) ")
            if [[ $response =~ ^[Yy]$ ]]; then
                functions_to_add+="
# rkit completion for bash
function _rkit_completion {
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

    # Add the functions to the config file
    echo "$functions_to_add" >> "$CONFIG_FILE"
    echo "Functions have been added to $CONFIG_FILE"
    echo "Please restart your shell or run 'source $CONFIG_FILE' to apply the changes."
}

# Main execution
add_to_shell_config 