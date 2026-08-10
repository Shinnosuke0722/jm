#!/opt/homebrew/bin/bash
# Run one SDKMAN command with no profile files. SDKMAN_DIR must name an isolated copy.
set -eo pipefail

: "${SDKMAN_DIR:?SDKMAN_DIR must point at an isolated SDKMAN installation}"
source "$SDKMAN_DIR/bin/sdkman-init.sh"
sdk "$@"
