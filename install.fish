#!/usr/bin/env fish

# Script to install rkit shell functions
# This script adds useful functions to your fish functions directory
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

# Create functions directory if it doesn't exist
set functions_dir "$HOME/.config/fish/functions"
if not test -d "$functions_dir"
    mkdir -p "$functions_dir"
    echo "Created functions directory at $functions_dir"
end

# Function to create individual function files
function create_function_file
    set function_name $argv[1]
    set function_content $argv[2]
    set function_file "$functions_dir/$function_name.fish"

    # Check if function file already exists
    if test -f "$function_file"
        echo "$function_name function is already installed at $function_file"
        return
    end

    echo "The following function will be created at $function_file:"
    echo
    echo "$function_content"
    echo
    read -l -P "Do you want to install the $function_name function? (y/N) " reply
    if test "$reply" = "y" -o "$reply" = "Y"
        echo "$function_content" > "$function_file"
        echo "Created $function_name function at $function_file"
    end
end

# Create clone function
create_function_file "clone" "function clone
    rkit clone \$argv
end"

# Create cdc function
create_function_file "cdc" "function cdc
    set -l query \$argv[1]
    if test -n \"\$query\"
        cd (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")
    else
        cd (rkit ls | fzf --preview 'rkit view {}')
    end
end"

# Create edit function
create_function_file "edit" "function edit
    set -l query \$argv[1]
    if test -n \"\$query\"
        code (rkit ls | fzf --preview 'rkit view {}' --query \"\$query\")
    else
        code (rkit ls | fzf --preview 'rkit view {}')
    end
end"

# Create completions file
set completions_file "$functions_dir/rkit-completions.fish"
if not test -f "$completions_file"
    echo "The following completions will be created at $completions_file:"
    echo
    echo "# rkit completions"
    echo "complete -c edit -a '(rkit ls)'"
    echo "complete -c cdc -a '(rkit ls)'"
    echo
    read -l -P "Do you want to install the completions? (y/N) " reply
    if test "$reply" = "y" -o "$reply" = "Y"
        echo "# rkit completions
complete -c edit -a '(rkit ls)'
complete -c cdc -a '(rkit ls)'" > "$completions_file"
        echo "Created completions at $completions_file"
    end
else
    echo "Completions are already installed at $completions_file"
end

echo "Installation complete!"
echo "Please restart your shell or run 'source $HOME/.config/fish/config.fish' to use the new functions" 