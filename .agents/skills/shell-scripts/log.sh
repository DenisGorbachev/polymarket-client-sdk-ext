function info() {
  echo "[I] $*" >&2
}

function warn() {
  echo "[W] $*" >&2
}

function error() {
  echo "[E] $*" >&2
  return 1
}
