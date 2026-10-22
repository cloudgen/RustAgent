#!/bin/sh
# Maintainer verbs for the RustAgent crate. rust-agent flags are not accepted here.
# upload does not run cargo publish.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
cd "$ROOT"

PROJECT="RustAgent"

out_err() {
    printf '%s\n' "$1" >&2
}

read_version() {
    sed -n 's/^version = "\([^"]*\)".*/\1/p' Cargo.toml | head -n 1
}

VERSION=$(read_version 2>/dev/null || true)

need_version() {
    if [ -z "$VERSION" ]; then
        out_err "ERROR: Cannot read the package version from Cargo.toml."
        out_err "   Next: set package.version, then ./build.sh version"
        exit 1
    fi
}

if [ -n "$VERSION" ]; then
    printf '%s\n' "$PROJECT build tool (v$VERSION)"
else
    printf '%s\n' "$PROJECT build tool (version unread)"
fi
printf '%s\n' "========================================"

show_help() {
    cat << EOF
Usage: ./build.sh <command>

Operational commands:
  help       Show this list (also: no command, -h, --help)
  version    Print the package version from Cargo.toml
  setup      Check that rustc and cargo are on PATH
  clean      Remove target/
  build      cargo build --release
  upload     Refuses. The registry command is cargo publish, and this script does not run it
  git        Ask for one commit message, then stage, commit, and push
  tag        Create and push annotated tag v${VERSION:-unread}
  release    clean, then build. Does not publish and does not tag
  all        Same chain as release
  test-install  cargo install --path . --force

Test command:
  test       Run tests/run.sh. No extra arguments.

Examples:
  ./build.sh version
  ./build.sh build
  ./build.sh test
  ./build.sh test-install
EOF
}

do_setup() {
    if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
        out_err "ERROR: rustc and cargo must be on PATH."
        out_err "   Next: install the Rust toolchain, then ./build.sh setup"
        exit 1
    fi
    rustc --version
    cargo --version
}

do_clean() {
    rm -rf target
    printf '%s\n' "Clean complete"
}

do_build() {
    if ! cargo build --release; then
        out_err "ERROR: Package build failed."
        out_err "   Next: ./build.sh setup"
        exit 1
    fi
    printf '%s\n' "Build complete -> target/release/"
}

do_upload() {
    out_err "ERROR: ./build.sh upload does not run cargo publish."
    out_err "   Next: cargo publish"
    exit 1
}

do_git() {
    if [ -t 0 ]; then
        printf '%s\n' "Enter commit message:"
        read -r message || message=""
    else
        IFS= read -r message || message=""
    fi
    if [ -z "$message" ]; then
        out_err "ERROR: Commit message is empty."
        out_err "   Next: ./build.sh git"
        exit 1
    fi
    git add .
    git commit -m "$message"
    git push
}

do_tag() {
    need_version
    TAG="v$VERSION"
    printf '%s\n' "Creating and pushing tag: $TAG"
    git tag -a "$TAG" -m "Release $TAG"
    git push origin "$TAG"
}

do_test_install() {
    need_version
    printf '%s\n' "Installing this checkout with cargo"
    if ! cargo install --path . --force; then
        out_err "ERROR: Local install failed."
        out_err "   Next: ./build.sh test-install"
        exit 1
    fi
}

cmd=${1:-}
case "$cmd" in
    -h|--help|help|"")
        show_help
        ;;
    version)
        need_version
        printf '%s\n' "$PROJECT build tool (v$VERSION)"
        ;;
    setup)
        do_setup
        ;;
    clean)
        do_clean
        ;;
    build)
        do_build
        ;;
    upload)
        do_upload
        ;;
    git)
        do_git
        ;;
    tag)
        do_tag
        ;;
    test)
        shift
        if [ "$#" -ne 0 ]; then
            out_err "ERROR: ./build.sh test takes no extra arguments."
            out_err "   Next: ./build.sh test"
            exit 1
        fi
        if [ ! -f ./tests/run.sh ]; then
            out_err "ERROR: tests/run.sh is not in this checkout."
            out_err "   Next: add the suite runner, then ./build.sh test"
            exit 1
        fi
        ./tests/run.sh
        ;;
    test-install)
        shift
        if [ "$#" -ne 0 ]; then
            out_err "ERROR: ./build.sh test-install takes no extra arguments."
            out_err "   Next: ./build.sh test-install"
            exit 1
        fi
        do_test_install
        ;;
    release|all)
        need_version
        do_clean
        do_build
        ;;
    *)
        out_err "ERROR: Unknown command: $cmd"
        show_help
        exit 1
        ;;
esac

printf '%s\n' "Done."
