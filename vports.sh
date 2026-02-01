#!/bin/bash
# Creates virtual serial ports from config/ports.toml using socat
# Usage: ./vports.sh        - create loopback ports
#        ./vports.sh -k     - kill all socat processes

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_PATH="$PROJECT_DIR/config/ports.toml"

kill_ports() {
    echo "Killing socat processes..."
    pkill -f "socat.*pty.*link=/tmp/tty" 2>/dev/null
    sleep 0.5
    echo "Done"
}

create_ports() {
    if [[ ! -f "$CONFIG_PATH" ]]; then
        echo "Config not found: $CONFIG_PATH"
        exit 1
    fi

    # Extract paths from TOML (lines like: path = "/tmp/ttyV0")
    paths=$(grep -E '^\s*path\s*=' "$CONFIG_PATH" | sed 's/.*=\s*"\(.*\)"/\1/')

    if [[ -z "$paths" ]]; then
        echo "No ports found in config"
        exit 1
    fi

    echo "Creating virtual serial ports..."

    for path in $paths; do
        # Create echo loopback - anything written gets echoed back
        socat pty,raw,echo=1,link="$path" EXEC:'cat',pty,raw,echo=0 &
        sleep 0.1
        echo "  Created: $path"
    done

    sleep 0.3
    echo ""
    echo "Ports ready. Use './vports.sh -k' to kill them."
    ls -la /tmp/ttyV* 2>/dev/null
}

case "$1" in
    -k|--kill)
        kill_ports
        ;;
    -h|--help)
        echo "Usage: $0 [-k|--kill]"
        echo "  (no args)  Create virtual serial ports from config/ports.toml"
        echo "  -k         Kill all virtual serial ports"
        ;;
    *)
        create_ports
        ;;
esac
