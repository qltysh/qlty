#!/bin/sh

# This script is based on Bun (https://github.com/oven-sh/bun)
# https://github.com/oven-sh/bun/blob/main/src/cli/install.sh
# MIT License

set -eu

# Reset
Color_Off=''

# Regular Colors
Red=''
Green=''
Dim='' # White

# Bold
Bold_White=''
Bold_Green=''

if [ -t 1 ]; then
    # Reset
    Color_Off=$(printf '\033[0m') # Text Reset

    # Regular Colors
    Red=$(printf '\033[0;31m')   # Red
    Green=$(printf '\033[0;32m') # Green
    Dim=$(printf '\033[0;2m')    # White

    # Bold
    Bold_Green=$(printf '\033[1;32m') # Bold Green
    Bold_White=$(printf '\033[1m')    # Bold White
fi

error() {
    printf '%berror%b: %s\n' "${Red}" "${Color_Off}" "$*" >&2
    exit 1
}

info() {
    printf '%b%s %b\n' "${Dim}" "$*" "${Color_Off}"
}

info_bold() {
    printf '%b%s %b\n' "${Bold_White}" "$*" "${Color_Off}"
}

success() {
    printf '%b%s %b\n' "${Green}" "$*" "${Color_Off}"
}

command -v xz >/dev/null ||
    error 'xz is required to install qlty'

if [ "$#" -gt 0 ]; then
    error 'Too many arguments, none are allowed'
fi

# Hardcoding the asset_id for now, which means this will always install a specific
# version of qlty, rather than the latest
#
# This is a workaround because the qltysh/qlty repository is currently private,
# which means the only way to download assets is using the GitHub API, rather than
# browser download URLs.
#
# In theory, we could lookup the asset_id for the latest release by querying the API.
# However, this would require parsing and searching through JSON output, and I don't
# know of a good universally portable way to do that without requiring/installing
# additional software. (macOS no longer ships with python.)
#
# So we settle for this for now, and after installation users can do `qlty upgrade`
case $(uname -ms) in
    'Darwin x86_64')
        target=x86_64-apple-darwin
        ;;
    'Darwin arm64')
        target=aarch64-apple-darwin
        ;;
    'Linux aarch64' | 'Linux arm64')
        target=aarch64-unknown-linux-gnu
        ;;
    'Linux x86_64' | *)
        target=x86_64-unknown-linux-gnu
        ;;
esac

if [ "$target" = x86_64-apple-darwin ]; then
    # Is this process running in Rosetta?
    # redirect stderr to devnull to avoid error message when not running in Rosetta
    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null)" = 1 ]; then
        target=aarch64-apple-darwin
        info "Your shell is running in Rosetta 2. Downloading qlty for $target instead"
    fi
fi

case "$target" in
    *linux*)
        # Store ldd --version output
        ldd_version_output=$(ldd --version 2>&1) || true

        # Check if the output contains 'musl'
        if echo "$ldd_version_output" | grep -q 'musl'; then
            target=$(echo "$target" | sed 's/-gnu/-musl/')
        fi

        # Extract GLIBC version
        glibc_version=$(echo "$ldd_version_output" | awk '/ldd/{print $NF}')

        # Provide default values to avoid unbound variable errors
        major=0
        minor=0

        # Extract major and minor version numbers
        if [ -n "$glibc_version" ]; then
            major=$(echo "$glibc_version" | cut -d. -f1)
            minor=$(echo "$glibc_version" | cut -d. -f2)
        fi

        # Check if the GLIBC version is less than 2.32
        if [ "$major" -lt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -lt 32 ]; }; then
            target=$(echo "$target" | sed 's/-gnu/-musl/')
        fi
        ;;
esac

exe_name=qlty

version=latest
if [ -n "${QLTY_VERSION-}" ]; then
    version="v${QLTY_VERSION#v}"
fi

url_prefix=${QLTY_INSTALL_URL-https://qlty-releases.s3.amazonaws.com/qlty}
qlty_uri="$url_prefix/$version/qlty-$target.tar.xz"

install_env=QLTY_INSTALL
bin_env=\$$install_env/bin

install_dir=$HOME/.qlty
bin_dir=${QLTY_INSTALL_BIN_PATH-$install_dir/bin}
exe=$bin_dir/qlty

download_dir=$install_dir/downloads
download=$download_dir/qlty.tar.xz

if [ ! -d "$bin_dir" ]; then
    mkdir -p "$bin_dir" ||
        error "Failed to create directory \"$bin_dir\""
fi

if [ ! -d "$download_dir" ]; then
    mkdir -p "$download_dir" ||
        error "Failed to create directory \"$download_dir\""
fi

if command -v curl >/dev/null 2>&1; then
    curl --fail --location --progress-bar --output "$download" "$qlty_uri" ||
        error "Failed to download qlty from \"$qlty_uri\""
elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$download" "$qlty_uri" ||
        error "Failed to download qlty from \"$qlty_uri\""
else
    error 'curl or wget is required to install qlty'
fi

mkdir -p "$download_dir/qlty-$target" ||
    error "Failed to create directory \"$download_dir/qlty-$target\""

tar -xpJf "$download" --strip-components=1 -C "$download_dir/qlty-$target" ||
    error 'Failed to extract qlty'

mv "$download_dir/qlty-$target/$exe_name" "$exe" ||
    error 'Failed to move extracted qlty to destination'

chmod +x "$exe" ||
    error 'Failed to set permissions on qlty executable'

rm -r "$download_dir/qlty-$target" "$download"

tildify() {
    if [ "${1#"$HOME"/}" != "$1" ]; then
        replacement='~'
        echo "$replacement/${1#"$HOME"/}"
    else
        echo "$1"
    fi
}

success "qlty was installed successfully to $Bold_Green$(tildify "$exe")"

event_payload=$(cat <<EOF
  {
    "event": "CLI Installed",
    "properties": {
      "Target": "$target",
      "Environment": "${AWS_EXECUTION_ENV:-LOCAL}",
      "CI": "${CI:-false}",
      "Installer": "install.sh"
    },
    "anonymousId": "ef2de8ab98bd4b85d36e62e7323345b2"
  }
EOF
)

if command -v curl >/dev/null 2>&1; then
    curl --request POST \
    --url https://cdp.customer.io/v1/track \
    --header 'Authorization: Basic MTc2YzFjYzYyYTdmN2UzOTczMDI6'\
    --header 'content-type: application/json' \
    -d "$event_payload" \
    >/dev/null 2>&1
fi

if command -v qlty >/dev/null; then
    # Install completions, but we don't care if it fails
    $exe completions --install >/dev/null 2>&1 || :

    echo "Run 'qlty' to get started"
    exit
fi

refresh_command=''

tilde_bin_dir=$(tildify "$bin_dir")
quoted_install_dir="$(printf '%s' "$install_dir" | sed 's/"/\\"/g')"
quoted_install_dir="\"$quoted_install_dir\""

case "$quoted_install_dir" in
    "\"$HOME"/*)
        quoted_install_dir=$(printf '%s' "$quoted_install_dir" | sed "s|\"$HOME/|\"\$HOME/|")
        ;;
esac

echo

[ -z "${QLTY_NO_MODIFY_PATH-}" ] && case $(basename "${SHELL:-sh}") in
    fish)
        # Install completions, but we don't care if it fails
        SHELL=fish $exe completions --install >/dev/null 2>&1 || :
        fish_config=$HOME/.config/fish/config.fish
        tilde_fish_config=$(tildify "$fish_config")
        mkdir -p "$(dirname "$fish_config")"
        {
            printf '\n# qlty\n'
            echo "set --export $install_env $quoted_install_dir"
            echo "set --export PATH $bin_env \$PATH"
        } >>"$fish_config"
        info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_fish_config\""
        refresh_command="source $tilde_fish_config"
        ;;
    zsh)
        # Install completions, but we don't care if it fails
        SHELL=zsh $exe completions --install >/dev/null 2>&1 || :
        zsh_config=$HOME/.zshrc
        tilde_zsh_config=$(tildify "$zsh_config")
        {
            printf '\n# qlty\n'
            echo "export $install_env=$quoted_install_dir"
            echo "export PATH=\"$bin_env:\$PATH\""
        } >>"$zsh_config"
        info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_zsh_config\""
        refresh_command="exec $SHELL"
        ;;
    bash)
        # Install completions, but we don't care if it fails
        SHELL=bash $exe completions --install >/dev/null 2>&1 || :
        bash_configs="$HOME/.bash_profile $HOME/.bashrc"
        if [ -n "${XDG_CONFIG_HOME:-}" ]; then
            bash_configs="$XDG_CONFIG_HOME/.bash_profile $XDG_CONFIG_HOME/.bashrc $XDG_CONFIG_HOME/bash_profile $XDG_CONFIG_HOME/bashrc $bash_configs"
        fi

        for bash_config in $bash_configs; do
            tilde_bash_config=$(tildify "$bash_config")

            use_bash_config=false
            if [ -w "$bash_config" ]; then
                use_bash_config=true
            elif [ "$(basename "$bash_config")" = ".bashrc" ]; then
                use_bash_config=true
            fi

            if [ "$use_bash_config" = true ]; then
                {
                    printf '\n# qlty\n'
                    echo "export $install_env=$quoted_install_dir"
                    echo "export PATH=$bin_env:\$PATH"
                } >>"$bash_config"
                info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_bash_config\""
                refresh_command="source $bash_config"
                break
            fi
        done
        ;;
    sh)
        profile=$HOME/.profile
        tilde_profile=$(tildify "$profile")

        {
            printf '\n# qlty\n'
            echo "export $install_env=$quoted_install_dir"
            echo "export PATH=\"$bin_env:\$PATH\""
        } >>"$profile"

        info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_profile\""
        refresh_command="source $profile"
        ;;
    *)
        echo 'Manually add the directory to your shell configuration file:'
        info_bold "  export $install_env=$quoted_install_dir"
        info_bold "  export PATH=\"$bin_env:\$PATH\""
        ;;
esac

# Transparent support for GitHub CI
if [ -n "${GITHUB_PATH:-}" ]; then
    echo "$bin_env" >> "$GITHUB_PATH"
fi

echo
info "To get started, run:"
echo

if [ -n "${refresh_command:-}" ]; then
    info_bold "  $refresh_command"
fi
info_bold "  qlty --help"
