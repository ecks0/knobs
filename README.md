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
knobs 0.5.0

USAGE:
    knobs [OPTIONS] [ARGS]

OPTIONS:
    -q, --quiet                    Do not print tables
        --show-cpu                 Show cpu tables
        --show-rapl                Show rapl table
        --show-drm                 Show drm tables
    -c, --cpu <IDS>                Target cpu ids
    -o, --cpu-on <BOOL>            Set cpu online or offline
    -g, --cpu-gov <STR>            Set cpu governor
    -n, --cpu-min <MHZ>            Set cpu min freq in megahertz
    -x, --cpu-max <MHZ>            Set cpu max freq in megahertz
        --cpu-epb <0..=15>         Set cpu epb
        --cpu-epp <STR>            Set cpu epp
    -P, --rapl-package <INT>       Target rapl package
    -S, --rapl-subzone <INT>       Target rapl subzone
    -C, --rapl-constraint <INT>    Target rapl constraint
    -L, --rapl-limit <WATTS>       Set rapl power limit in watts
    -W, --rapl-window <μS>         Set rapl power window in microseconds
        --i915 <IDS>               Target i915 drm ids or bus ids
        --i915-min <MHZ>           Set i915 min freq in megahertz
        --i915-max <MHZ>           Set i915 max freq in megahertz
        --i915-boost <MHZ>         Set i915 boost freq in megahertz
        --nvml <IDS>               Target nvml drm ids or bus ids
        --nvml-gpu-min <MHZ>       Set nvml min gpu freq in megahertz
        --nvml-gpu-max <MHZ>       Set nvml max gpu freq in megahertz
        --nvml-power <WATTS>       Set nvml device power limit in watts
    -h, --help                     Prints help information

ARGS:
    <ARGS>
```

## Running

Knobs may be called with multiple groups of arguments. Argument groups
are separated by `--`.
```
# knobs --cpu 1.. --online true -- --cpu .. --cpu-gov schedutil
        │                       │  │
        └ first argument group  │  └ second argument group
                                │
                                └ argument group separator
```
```
# knobs -P 0 -C 0 --rapl-limit 7 -- -P 0 -C 1 --rapl-limit 14
        │                        │  │
        └ first argument group   │  └ second argument group
                                 │
                                 └ argument group separator
```
All argument groups are parsed, and device IDs verified, before values are
written. Any i/o error will cause an immediate exit with error status.

Most values can be displayed as a non-root user, but setting values
typically requires running with root privileges.

```
$ whoami
notroot
$ knobs -c .. -g schedutil
Error: write: /sys/devices/system/cpu/cpufreq/policy0/scaling_governor: Permission denied (os error 13)
$
```

## Output

With no arguments, knobs will show tables for all supported devices detected on the system.

With arguments, after writing device values, knobs will show tables for devices related to
the given arguments.

Particular table topics may be displayed with the `--show-*` arguments, the use of which
overrides the default table handling behavior descibed above.

Output may be silenced with `--quiet`.

Example output:
```
 CPU   Online  Governor     Cur      Min      Max      Min limit  Max limit
 ----  ------  --------     ----     ----     ----     ---------  ---------
 0     •       performance  4.6 GHz  800 MHz  4.7 GHz  800 MHz    4.7 GHz
 1     true    performance  4.6 GHz  800 MHz  4.7 GHz  800 MHz    4.7 GHz
 2     true    performance  4.7 GHz  800 MHz  4.7 GHz  800 MHz    4.7 GHz
 3     true    performance  4.3 GHz  800 MHz  4.7 GHz  800 MHz    4.7 GHz

 CPU   Available governors
 ----  -------------------
 all   performance powersave

 CPU   EP bias  EP preference
 ----  -------  -------------
 all   6        balance_performance

 CPU   Available EP preferences
 ----  ------------------------
 all   default performance balance_performance balance_power power

 RAPL  Zone name  Long lim  Short lim  Long win     Short win  Usage
 ----  ---------  --------  ---------  --------     ---------  -----
 0     package-0  28 W      35 W       27983872 us  2440 us    3.8 W
 0:0   core       0 W       •          976 us       •          1.4 W
 0:1   uncore     0 W       •          976 us       •          27.5 mW
 1     psys       0 W       0 W        27983872 us  976 us     98.9 mW

 DRM   Driver  Actual   Req'd    Min      Max      Boost    Min lim  Max lim
 ----  ------  ------   -----    ----     ----     -----    -------  -------
 0     i915    100 MHz  300 MHz  100 MHz  1.4 GHz  1.4 GHz  100 MHz  1.4 GHz

 DRM   Driver  GPU cur  GPU lim  Power cur  Power lim  Min lim  Max lim
 ----  ------  -------  -------  ---------  ---------  -------  -------
 1     nvidia  300 MHz  2.2 GHz  12 W       260 W      100 W    325 W
 ```
