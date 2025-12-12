set dotenv-load

# List available commands
default:
    @just -l

alias u := update
alias d := dev
alias t := test
alias t-u := test-update

ROLLUP := "rollup client/index.js --file assets/static/index.min.js --format iife"
TAILWIND := "./tailwindcss -i ./src/styles/styles.css -o ./assets/styles.css"

# Runs the Tailwind binary in watch mode
[no-exit-message, private]
run-tailwind:
    #!/bin/bash
    echo "Starting the Tailwind binary."
    {{TAILWIND}} --watch

# Builds and minifies the CSS with the Tailwind binary
[private]
build-tailwind:
    #!/bin/bash
    echo "minifying css"
    {{TAILWIND}} --minify

# Install the latest tailwind binary in your system
[private]
install-tailwind:
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
[no-exit-message, private]
run-axum:
    #!/bin/bash
    echo "Starting the Axum server."

    export API_TOKEN=$API_TOKEN

    # Start cargo watch in the background
    cargo watch -w src -x run

# Runs rollup in watch mode.
[no-exit-message, private]
run-rollup:
    #!/bin/bash
    echo "Starting rollup."
    {{ROLLUP}} --watch --watch.exclude "src/**" --no-watch.clearScreen

# Builds and minifies the JS with rollup 
[private]
build-rollup:
    #!/bin/bash
    echo "building JS"
    {{ROLLUP}} -p @rollup/plugin-terser 

# Updates the requested versions of assets found in the 
# base.html template to bust cached versions of old assets.
[private]
bump-assets:
    #!/bin/bash
    echo "bumping static assets version numbers in base.html"
    target/release/bump-versions

# Builds all the static assets and bumps their versions
[group('Build')]
build: build-tailwind build-rollup bump-assets

# Run the axum server, rollup, and tailwind binary in watch mode so updates
# will automatically be reflected. On exit, will minify tailwind's css and js.
#
# Install Just and run with `just dev`
[group('Development')]
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
    just test && cargo set-version --bump patch

# Runs the tests, writes new snapshots
[group('Test')]
test:
    #!/bin/bash
    # unseen: writes new snapshots and writes .snap.new for exisiting
    INSTA_UPDATE=unseen cargo t --features mock-time && node --test

# Runs the tests, and updates all snapshots
[group('Test')]
test-update:
    #!/bin/bash
    # always: overwrites old snapshot files with new ones unasked
    INSTA_UPDATE=always cargo t --features mock-time

# Installs rollup and the terser plugin globally
[private]
install-rollup:
    #!/bin/bash
    echo "Installing rollup"
    npm install --global rollup
    echo "Installing rollup terser plugin"
    npm install --global @rollup/plugin-terser

# Compiles the helper binary to bump static asset versions in base.html
[private]
install-bump-versions:
    #!/bin/bash 
    FILE=./target/release/bump-versions

    if [ ! -f "$FILE" ]
    then
        echo "Building bump-versions"
        cargo build --bin bump-versions --release
    fi

[private, macos]
install-yq:
    #!/bin/bash
    brew install yq

[private, linux]
install-yq:
    #!/bin/bash
    sudo wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq \
        && chmod +x /usr/local/bin/yq

# Installs the projects dependencies required to run the project, other than just
[group('Installation')]
install:
    #!/bin/bash
    if command -v cargo &> /dev/null; then
        echo "Cargo found, skipping Rust install"
    else
        # install Rust
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    fi

    if command yq --version &> /dev/null; then
        echo "yq found, skipping install"
    else
        just install-yq
    fi

    if command cargo watch --version &> /dev/null; then
        echo "Cargo watch found, skipping install"
    else
        # install cargo watch
        cargo install cargo-watch
        cargo install cargo-edit
    fi

    just install-bump-versions

    just install-tailwind

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
        fnm use --install-if-missing 24

        just install-rollup
    fi

# Builds the docker image
[private]
docker-build:
    docker buildx build --platform linux/arm64/v8 --tag gathering_surf --file Dockerfile.prod .

[private]
docker-deploy:
    DOCKER_HOST="ssh://austin@cluster.local" docker compose up -d

[private]
docker-local:
    docker build --tag gathering_surf --file Dockerfile.local . && docker compose up -d

# Builds the x86 docker image and tags it with the registry location
[group('Build')]
build-kube:
    docker build --tag registry:5001/gathering_surf:${TAG:-latest} --file Dockerfile.local .

# Updates the cluster's registry with the latest image
[private]
upload-kube:
    #!/bin/bash
    set -euo pipefail

    # Build the image
    just build-kube

    # Launch the tunnel in background
    # Map port 5001 to node0:80
    ssh -L 5001:10.110.129.160:80 austin@192.168.1.121 -p 222 -N &
    TUNNEL_PID=$!          # capture the background PID

    # Close the tunnel when the process completes or fails
    trap 'echo "Stopping tunnel…"; kill "$TUNNEL_PID" 2>/dev/null || true' EXIT INT TERM

    # Wait for the tunnel to be ready
    echo -n "Waiting for local port 5001 to be ready"
    while ! nc -z localhost 5001; do
        sleep .25
        printf "."
    done
    echo "Tunnel started (PID $TUNNEL_PID) – local port 5001 → 10.110.129.160:80"

    # Push the image to the registry
    # Requires that `/etc/hosts` has registry 127.0.0.1
    # The hostname needs to be registry becuase that's how the ingress in the 
    # kube cluster knows to route it to the service 
    # i.e. in the cluster itself `curl -H "Host: registry"` is required
    # Docker connects to localhost:5001 and sends Host: registry:5001.
    echo "Pushing image to registry"
    docker push registry:5001/gathering_surf:$TAG

# Updates the cluster's image and deployment file, then applies it.
[group('Deploy')]
deploy:
    #!/bin/bash
    export HOST="austin@192.168.1.121"
    export PORT="222"
    export TAG=$(yq '.package.version' Cargo.toml)

    # Upload the latest build of the image to the internal registry, then
    # update the tag in the kube config file, send it to node0, then apply it.
    # User must be in the deploygrp on node0 to be able to create files there!
    just upload-kube \
        && yq eval -i 'select(.metadata.name=="gathering-surf-server" and .kind=="Deployment").spec.template.spec.containers[].image = "10.108.202.38:5000/gathering_surf:'$TAG'"' kube-deployment.yaml \
        && scp -P "$PORT" ./kube-deployment.yaml $HOST:/opt/deploys/gathering_surf.yaml \
        && ssh -p "$PORT" $HOST "kubectl apply -f /opt/deploys/gathering_surf.yaml"
