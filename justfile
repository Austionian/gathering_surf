set dotenv-load

# List available commands
default:
    just -l

alias u := update
alias d := dev

# Script to run the Tailwind binary in watch mode
run-tailwind:
    #!/bin/bash
    echo "Starting the Tailwind binary."
    ./tailwindcss -i ./src/styles/styles.css -o ./assets/styles.css --watch

# Script to build and minify the Tailwind binary
build-tailwind:
    #!/bin/bash
    echo -e "\nMinifying css"
    sh -c './tailwindcss -i ./src/styles/styles.css -o ./assets/styles.css --minify'

# Script to run the axum server in watch mode.
run-axum:
    #!/bin/bash
    echo "Starting the Axum server."

    export API_TOKEN=$API_TOKEN

    # Start cargo watch in the background
    sh -c 'cargo watch -w src -w templates -x run &'

run-rollup:
    #!/bin/bash
    echo "Starting rollup."
    
    sh -c 'rollup client/index.js --file assets/static/index.min.js --format iife -p @rollup/plugin-terser --watch --watch.exclude "src/**" --no-watch.clearScreen &'

# Script to run the axum server and tailwind binary in watch mode so updates
# will automatically be reflected. On exit, will minify tailwind's css.
#
# Install Just and run with `just dev`
dev:
    #!/bin/bash
    minify() {
        just build-tailwind
    }

    # Add a trap to run the minify function before exiting
    trap "minify; kill 0" SIGINT

    open 'http://127.0.0.1:8080'

    just run-axum
    
    just run-rollup

    just run-tailwind
    TAILWIND_PID=$!

    wait $TAILWIND_PID

# Update dependencies and run the tests.
update:
    #!/bin/bash
    cargo update
    echo $'Dependencies updated!\n'
    cargo clippy
    cargo test

# Installs the projects dependencies required to run the project, other
# than Just obviously. MacOS only.
install:
    #!/bin/bash
    if command -v cargo &> /dev/null; then
        echo "Cargo found, skipping Rust install"
    else
        # install Rust
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    fi

    # install the Tailwind binary
    curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64
    chmod +x tailwindcss-macos-arm64
    mv tailwindcss-macos-arm64 tailwindcss

    # check if npm is available
    if command -v node &> /dev/null; then
        # install rollup and minification plugin
        npm install --global rollup
        npm install --global @rollup/plugin-terser
    else
        echo "npm not found. Installing fnm and node."
        # installs fnm (Fast Node Manager)
        curl -fsSL https://fnm.vercel.app/install | bash

        # activate fnm
        source ~/.bashrc

        # download and install Node.js
        fnm use --install-if-missing 22

        # install rollup and minification plugin
        npm install --global rollup
        npm install --global @rollup/plugin-terser
    fi
        

