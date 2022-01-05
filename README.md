# knobs

A command-line utility for controlling Linux performance settings.

| Topic        | Supported values                            |
| ------------ | ------------------------------------------- |
| cpu          | on/offline, governor, frequencies, epb/epp  |
| rapl         | power limit/time window per zone/constraint |
| i915         | min/max/boost frequencies                   |
| nvml         | gpu clock min/max frequency, power limit    |

## Help

Run with `-h` for short help or `--help` for long help.

Short help:
```
knobs 0.5.1

USAGE:
    knobs [OPTIONS] [-- <ARGS>...]

ARGS:
    <ARGS>...    Additional option groups

OPTIONS:
    -q, --quiet                    Do not print tables
        --show-cpu                 Print cpu tables
        --show-rapl                Print rapl table
        --show-drm                 Print drm tables
    -c, --cpu <IDS>                Target cpu ids
    -o, --cpu-on <BOOL>            Set cpu online or offline
    -g, --cpu-gov <STR>            Set cpu governor
    -n, --cpu-min <INT>            Set cpu min freq in megahertz
    -x, --cpu-max <INT>            Set cpu max freq in megahertz
        --cpu-epb <0..=15>         Set cpu epb
        --cpu-epp <STR>            Set cpu epp
    -P, --rapl-package <INT>       Target rapl package
    -S, --rapl-subzone <INT>       Target rapl subzone
    -C, --rapl-constraint <INT>    Target rapl constraint
    -L, --rapl-limit <FLOAT>       Set rapl power limit in watts
    -W, --rapl-window <INT>        Set rapl power window in microseconds
        --i915 <IDS>               Target i915 drm card indexes or bus ids
        --i915-min <INT>           Set i915 min freq in megahertz
        --i915-max <INT>           Set i915 max freq in megahertz
        --i915-boost <INT>         Set i915 boost freq in megahertz
        --nvml <IDS>               Target nvml drm card indexes or bus ids
        --nvml-gpu-min <INT>       Set nvml min gpu freq in megahertz
        --nvml-gpu-max <INT>       Set nvml max gpu freq in megahertz
        --nvml-power <FLOAT>       Set nvml device power limit in watts
    -h, --help                     Print help information
```

## Running

Knobs may be called with multiple groups of arguments. Argument groups
are separated by `--`.
```
# knobs -P 0 -C 0 --rapl-limit 7 -- -P 0 -C 1 --rapl-limit 14
        │                        │  │
        └ first argument group   │  └ second argument group
                                 │
                                 └ argument group separator
```
All argument groups are parsed and device IDs verified before values are
written. Any i/o error will cause an immediate exit with error status.

The `--quiet` and `--show-*` flags are parsed from the first argument group only.

Tables can be printed as a non-root user, but setting values typically
requires running with root privileges.

```
$ whoami
notroot
$ knobs --show-rapl
 RAPL  Zone name  Long lim  Short lim  Long win     Short win  Usage
 ----  ---------  --------  ---------  -----------  ---------  -----
 0     package-0  95 W      131 W      27983872 μs  2440 μs    •
 0:0   dram       0 W       •          976 μs       •          •
$ knobs -c .. -g schedutil
Group 1: error: write: /sys/devices/system/cpu/cpufreq/policy0/scaling_governor: Permission denied (os error 13)
$
```

## Output

With no arguments, knobs will show tables for all supported devices detected on the system.

With arguments, after writing device values, knobs will show tables for devices related to
the given arguments.

Particular table topics may be displayed with the `--show-*` arguments, the use of which
overrides the default table handling behavior descibed above.

Output may be silenced with `--quiet`.

Set `KNOBS_LOG=trace` to see what's happening under the hood.

Example output:

**i7-1160G7**
```
$ knobs
 CPU  Online  Governor   Cur      Min      Max      Min lim  Max lim
 ---  ------  ---------  -------  -------  -------  -------  -------
 0    •       powersave  1.2 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 1    true    powersave  1.3 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 2    true    powersave  1.3 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 3    true    powersave  1.1 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 4    true    powersave  1.5 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 5    true    powersave  1.4 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 6    true    powersave  1.5 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz
 7    true    powersave  1.5 GHz  400 MHz  4.4 GHz  400 MHz  4.4 GHz

 CPU  Available governors
 ---  ---------------------
 all  performance powersave

 CPU  EP bias  EP preference
 ---  -------  -------------------
 all  6        balance_performance

 CPU  Available EP preferences
 ---  -----------------------------------------------------------
 all  default performance balance_performance balance_power power

 RAPL  Zone name  Long lim  Short lim  Long win     Short win  Usage
 ----  ---------  --------  ---------  -----------  ---------  ------
 0     package-0  15 W      40 W       27983872 μs  2440 μs    2.4 W
 0:0   core       0 W       •          976 μs       •          400 mW
 0:1   uncore     0 W       •          976 μs       •          100 mW

 DRM  Bus  Bus id
 ---  ---  ------------
 0    pci  0000:00:02.0

 DRM  Driver  Actual   Req'd    Min      Max      Boost    Min lim  Max lim
 ---  ------  -------  -------  -------  -------  -------  -------  -------
 0    i915    100 MHz  350 MHz  100 MHz  1.1 GHz  1.1 GHz  100 MHz  1.1 GHz
```

**i7-8700k**
```
$ knobs
 CPU  Online  Governor     Cur      Min      Max      Min lim  Max lim
 ---  ------  -----------  -------  -------  -------  -------  -------
 0    •       performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 1    true    performance  4.4 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 2    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 3    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 4    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 5    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 6    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 7    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 8    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 9    true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 10   true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz
 11   true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz  4.7 GHz

 CPU  Available governors
 ---  ---------------------------------------------------------------
 all  conservative ondemand userspace powersave performance schedutil

 intel_pstate: passive

 RAPL  Zone name  Long lim  Short lim  Long win     Short win  Usage
 ----  ---------  --------  ---------  -----------  ---------  -----
 0     package-0  95 W      131 W      27983872 μs  2440 μs    6.8 W
 0:0   dram       0 W       •          976 μs       •          1.4 W

 DRM  Bus  Bus id
 ---  ---  ------------
 0    pci  0000:02:00.0

 DRM  Driver  GPU cur  GPU lim  Power cur  Power lim  Min lim  Max lim
 ---  ------  -------  -------  ---------  ---------  -------  -------
 0    nvidia  1.4 GHz  2.2 GHz  43.1 W     260 W      100 W    325 W
```