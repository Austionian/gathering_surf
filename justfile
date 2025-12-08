# List available commands
default:
    just -l

alias u := update
alias d := dev
alias t := test
alias t-u := test-update

ROLLUP := "rollup client/index.js --file assets/static/index.min.js --format iife"
TAILWIND := "./tailwindcss -i ./src/styles/styles.css -o ./assets/styles.css"

# Runs the Tailwind binary in watch mode
[group('Development')]
[no-exit-message]
run-tailwind:
    #!/bin/bash
    echo "Starting the Tailwind binary."
    {{TAILWIND}} --watch

# Builds and minifies the CSS with the Tailwind binary
[group('Build')]
build-tailwind:
    #!/bin/bash
    echo "minifying css"
    {{TAILWIND}} --minify

# Install the latest tailwind binary in your system
[group('Set up')]
download-tailwind:
    #!/bin/bash
    if [ "$(uname)" == "Darwin" ]; then 
        curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64 
        chmod +x tailwindcss-macos-arm64 
        mv tailwindcss-macos-arm64 tailwindcss 
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then 
        if [ "$(uname -m)" == "x86_64" ]; then 
            curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 
            chmod +x tailwindcss-linux-x64 
            mv tailwindcss-linux-x64 tailwindcss 
        else 
            curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-arm64 
            chmod +x tailwindcss-linux-arm64
            mv tailwindcss-linux-arm64 tailwindcss
        fi
    fi

# Runs the axum server in watch mode.
[group('Development')]
[no-exit-message]
run-axum:
    #!/bin/bash
    echo "Starting the Axum server."

    # Start cargo watch in the background
    cargo watch -w src -x run

# Runs rollup in watch mode.
[group('Development')]
[no-exit-message]
run-rollup:
    #!/bin/bash
    echo "Starting rollup."
    {{ROLLUP}} --watch --watch.exclude "src/**" --no-watch.clearScreen

# Builds and minifies the JS with rollup 
[group('Build')]
build-rollup:
    #!/bin/bash
    echo "building JS"
    {{ROLLUP}} -p @rollup/plugin-terser 

# Updates the requested versions of assets found in the 
# base.html template to bust cached versions of old assets.
[group('Build')]
bump-assets:
    #!/bin/bash
    echo "bumping static assets version numbers in base.html"
    target/release/bump-versions

# Builds all the static assets and bumps their versions
[group('Build')]
build:
    #!/bin/bash
    just build-tailwind &
    just build-rollup &
    just bump-assets &
    wait
    echo "complete!"

# Run the axum server, rollup, and tailwind binary in watch mode so updates
# will automatically be reflected. On exit, will minify tailwind's css and js.
#
# Install Just and run with `just dev`
[group('Development')]
[no-exit-message]
dev:
    #!/bin/bash
    minify() {
        just build
    }

    # Add a trap to run the minify function before exiting
    trap "minify; kill 0" SIGINT

    open 'http://127.0.0.1:8080'

    just run-axum & just run-rollup & just run-tailwind

    TAILWIND_PID=$!

    wait $TAILWIND_PID

# Update dependencies and run the tests.
[group('Update')]
update:
    #!/bin/bash
    cargo update
    echo -e "Dependencies updated! \n"
    cargo clippy
    just test

# Runs the tests, writes new snapshots
[group('Update')]
test:
    #!/bin/bash
    # unseen: writes new snapshots and writes .snap.new for exisiting
    INSTA_UPDATE=unseen cargo t --features mock-time && node --test

# Runs the tests, and updates all snapshots
[group('Update')]
test-update:
    #!/bin/bash
    # always: overwrites old snapshot files with new ones unasked
    INSTA_UPDATE=always cargo t --features mock-time

# Installs rollup and the terser plugin globally
[group('Set up')]
install-rollup:
    #!/bin/bash
    echo "Installing rollup"
    npm install --global rollup
    echo "Installing rollup terser plugin"
    npm install --global @rollup/plugin-terser

# Compiles the helper binary to bump static asset versions in base.html
[group('Set up')]
install-bump-versions:
    #!/bin/bash 
    FILE=./target/release/bump-versions

    if [ ! -f "$FILE" ]
    then
        echo "Building bump-versions"
        cargo build --bin bump-versions --release
    fi

# Installs the projects dependencies required to run the project, other
# than Just obviously. MacOS only.
[group('Set up')]
install:
    #!/bin/bash
    if command -v cargo &> /dev/null; then
        echo "Cargo found, skipping Rust install"
    else
        # install Rust
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    fi

    if command cargo watch --version &> /dev/null; then
        echo "Cargo watch found, skipping install"
    else
        # install cargo watch
        cargo install cargo-watch
    fi

    just install-bump-versions && just download-tailwind

    # check if npm is available
    if command -v node &> /dev/null; then
        just install-rollup
    else
        echo "npm not found. Installing fnm and node."
        # installs fnm (Fast Node Manager)
        curl -fsSL https://fnm.vercel.app/install | bash

        # activate fnm
        source ~/.bashrc

        # download and install Node.js
        fnm use --install-if-missing 22

        just install-rollup
    fi

# Builds the docker image
[group('Build')]
docker-build:
    #!/bin/bash
    docker build --tag gathering_surf:$TAG --file Dockerfile.$HOST .

# Tells remote host to spin up containers
[group('Deploy')]
docker-deploy:
    #!/bin/bash
    DOCKER_HOST="ssh://austin@raspberry.tail473fdb.ts.net" docker compose -f ./docker-compose.yml up -d

# Build and transfer the gathering_surf image to our raspberry pi, then spin it
# up with docker compose.
[group('Deploy')]
deploy-qa:
    #!/bin/bash
    export TAG="qa"
    export HOST="raspberry"

    # Build the image
    # docker save - saves an image to tar archive, streamed to STDOUT
    # Then compress the tar archive using bzip2
    # pv is Pipe Viewer, shows to data flow between the piped commands
    # Then finally ssh into the raspberry pi and tell docker to load the streamed
    # image from STDIN.
    # Run the docker deploy command
    just docker-build \
        && docker save gathering_surf:$TAG | bzip2 | pv | ssh austin@raspberry.tail473fdb.ts.net docker load \
        && just docker-deploy 

# Run gathering surf locally with Docker Compose
[group('Deploy')]
docker-local:
    #!/bin/bash
    docker build --tag gathering_surf --file Dockerfile.local . && docker compose up -d

# Transfers the docker image to the pi and runs the deploy script
[group('Deploy')]
deploy:
    #!/bin/bash
    export TAG="latest"
    export HOST="cluster"

    just docker-build \
        && docker save gathering_surf:$TAG | bzip2 | ssh austin@$HOST.local docker load \
        && just docker-deploy

[group('Deploy')]
scale:
    #!/bin/bash
    export TAG="qa"
    export HOST="austin@raspberry.tail473fdb.ts.net"

    # Transfer the nginx conf
    scp ./nginx.conf $HOST:/home/austin/nginx.conf \
        && DOCKER_HOST="ssh://$HOST" docker compose -f ./docker-compose.replicas.yml up -d
