#!/bin/sh
unset LD_PRELOAD
TERMUX_EXEC_OPTOUT=1
export TERMUX_EXEC_OPTOUT
_g=__QUOTED__
_p=$(command -v proot 2>/dev/null || true)
if [ -z "${_p}" ]; then
  printf '%s\n' "proot is not on PATH, so grok cannot run on this Android host. Next: pkg install proot" >&2
  exit 127
fi
_k=""
if command -v grep >/dev/null 2>&1; then
  if "${_p}" --help 2>&1 | grep -q kill-on-exit; then
    _k="--kill-on-exit"
  fi
fi
_svc=""
if [ -n "${PREFIX:-}" ] && [ -f "${PREFIX}/etc/profile.d/start-services.sh" ]; then
  _svc="-b /dev/null:${PREFIX}/etc/profile.d/start-services.sh"
fi
_pflag=0
_nau=""
for _a in "$@"; do
  case "${_a}" in
    -p|--single|--prompt-file|--prompt-json) _pflag=1 ;;
    --no-auto-update) _nau="have" ;;
  esac
done
if [ "${_pflag}" -eq 1 ] && [ "${_nau}" != "have" ]; then
  _nau="--no-auto-update"
else
  _nau=""
fi
if [ "${_pflag}" -eq 1 ]; then
  # grok-cli proot-exit-reaper
  # grok-linux _exit()s; leftover runsvdir (bash -lc → start-services.sh)
  # stay PRoot tracees. Match args/exe, not truncated comm. Reap NEW only.
  _tmp="${TMPDIR:-${TMP:-/tmp}}"
  _out=$(mktemp "${_tmp}/grok-p-out.XXXXXX") || exit 1
  _err=$(mktemp "${_tmp}/grok-p-err.XXXXXX") || exit 1
  _before=$(mktemp "${_tmp}/grok-p-rsv.XXXXXX") || exit 1
  ps -eo pid= -o args= 2>/dev/null | awk '
    $2 ~ /\/runsvdir$/ || $2 == "runsvdir" { print $1 }
  ' | sort >"${_before}" 2>/dev/null || :
  trap '
    if [ -n "${_pid:-}" ]; then
      kill -TERM "${_pid}" 2>/dev/null || true
      sleep 1
      kill -KILL "${_pid}" 2>/dev/null || true
    fi
    rm -f "${_out}" "${_err}" "${_before}"
    exit 130
  ' INT TERM TSTP
  "${_p}" ${_k} ${_svc} __BIND__"${_g}" ${_nau} "$@" >"${_out}" 2>"${_err}" &
  _pid=$!
  (
    trap '' TSTP INT
    _seen=0
    _osize=0
    _idle_since=0
    _start=$(date +%s)
    while :; do
      _now=$(date +%s)
      if [ $((_now - _start)) -ge 180 ]; then
        break
      fi
      if ! kill -0 "${_pid}" 2>/dev/null; then
        exit 0
      fi
      _child=$(ps -eo pid= -o ppid= -o args= 2>/dev/null | awk -v col="${_pid}" '
        {
          pid=$1; ppid=$2; arg=$3
          p[pid]=ppid
          a[pid]=arg
        }
        END {
          for (i in a) {
            if (a[i] !~ /grok-linux/) continue
            c=i
            for (n=0; n<32; n++) {
              if (c == col) { print i; exit }
              if (!(c in p)) break
              if (p[c] == c || p[c] + 0 == 0) break
              c=p[c]
            }
          }
        }
      ')
      if [ -n "${_child}" ]; then
        _seen=1
      elif [ "${_seen}" -eq 1 ]; then
        break
      fi
      _sz=$(wc -c <"${_out}" 2>/dev/null | tr -d ' \t\n')
      _sz=${_sz:-0}
      if [ "${_sz}" -gt 0 ]; then
        if [ "${_sz}" -ne "${_osize}" ]; then
          _osize=${_sz}
          _idle_since=${_now}
        elif [ "${_idle_since}" -gt 0 ] && [ $((_now - _idle_since)) -ge 1 ]; then
          break
        fi
      fi
      sleep 1
    done
    ps -eo pid= -o args= 2>/dev/null | awk '
      $2 ~ /\/runsvdir$/ || $2 == "runsvdir" { print $1 }
    ' | sort | while read -r _rpid; do
      [ -n "${_rpid}" ] || continue
      if ! grep -qx "${_rpid}" "${_before}" 2>/dev/null; then
        kill "${_rpid}" 2>/dev/null || :
      fi
    done
    kill "${_pid}" 2>/dev/null || :
    kill -KILL "${_pid}" 2>/dev/null || :
    exit 0
  ) &
  _reaper=$!
  wait "${_reaper}" 2>/dev/null || :
  wait "${_pid}" 2>/dev/null || :
  trap - INT TERM TSTP
  if [ -s "${_err}" ]; then
    cat "${_err}" >&2
  fi
  if [ -s "${_out}" ]; then
    cat "${_out}"
    rm -f "${_out}" "${_err}" "${_before}"
    exit 0
  fi
  rm -f "${_out}" "${_err}" "${_before}"
  exit 1
fi
exec "${_p}" ${_k} ${_svc} __BIND__"${_g}" "$@"
