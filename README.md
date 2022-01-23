# knobs

Command-line utilities for controlling Linux performance settings.

Knobs provides several utilities in one binary, in the manner of busybox.
Utilities may be run via symlinks to the `knobs` binary, or as subcommands
to the `knobs` binary.

| Utility  | Subcommand | Function                                     |
| -------- | ---------- | -------------------------------------------- |
| kcpu     | cpu        | View or set cpu/cpufreq/intel_pstate values  |
| krapl    | rapl       | View or set intel_rapl values                |
| ki915    | i915       | View or set i915 values                      |
| knvml    | nvml       | View or set nvidia management library values |

## Utility symlinks

To install symlinks for utilities alongside the `knobs` binary,
run:
```
knobs install
```
To install symlinks to a particular directory, run:
```
knobs install /path/to/directory
```
Run utilities and subcommands with `-h` for short help or `--help` for long help.

## Argument groups

The cli accepts multiple argument groups, delimited by `--`. Argument
groups have three useful properties:

- All device ids are validated before any values are written.
- Any error will abort the entire invocation.
- Tables are printed once after all device values are written.

_Subcommand argument group example_

```bash
knobs \
    cpu -c .. -g schedutil -x 2000 -- \
    cpu -c 4.. -o false -- \
    rapl -p 0 -c 0 -l 7 -- \
    rapl -p 0 -c 1 -l 15
```
_Utility argument group example_
```
kcpu \
  -c .. -g schedutil -x 2000 -- \
  -c 4.. -o false

krapl \
  -p 0 -c 0 -l 7 -- \
  -p 0 -c 1 -l 15
```

- `knobs cpu` / `kcpu`
    - for all cpu ids, set governor to `schedutil` and max freq to 2000 mhz
    - for cpu ids 4 and up, set offline
- `knobs rapl` / `krapl`
    - for package 0, constraint 0, set power limit to 7 watts
    - for package 0, constraint 1, set power limit to 15 watts

## Utility reference

### kcpu

View or set cpu, cpufeq, and intel_pstate values.

```
$ kcpu -h
kcpu 0.6.0

USAGE:
    kcpu [OPTIONS]

OPTIONS:
    -c, --cpu <IDS>    Target cpu ids
    -o, --on <BOOL>    Set cpu online or offline
    -g, --gov <STR>    Set cpu governor
    -n, --min <INT>    Set cpu min freq in megahertz
    -x, --max <INT>    Set cpu max freq in megahertz
    -b, --epb <INT>    Set cpu epb
    -p, --epp <STR>    Set cpu epp
    -q, --quiet        Do not print tables
    -h, --help         Print help information
    -V, --version      Print version information
```
```
$ kcpu
 CPU  Online  Governor   Cur      Min      Max      Min lim  Max lim
 ---  ------  ---------  -------  -------  -------  -------  -------
 0    •       schedutil  1.1 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 1    true    schedutil  1.2 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 2    true    schedutil  1.5 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 3    true    schedutil  1.2 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 4    false   schedutil  1.1 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 5    false   schedutil  1.3 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 6    false   schedutil  1.1 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz
 7    false   schedutil  1.2 GHz  400 MHz  2.0 GHz  400 MHz  4.4 GHz

 CPU  Available governors
 ---  ---------------------------------------------------------------
 all  conservative ondemand userspace powersave performance schedutil

 intel_pstate: passive
```

### krapl

View or set intel_rapl values.

```
$ krapl -h
krapl 0.6.0

USAGE:
    krapl [OPTIONS]

OPTIONS:
    -p, --package <INT>       Target rapl package
    -s, --subzone <INT>       Target rapl subzone
    -c, --constraint <INT>    Target rapl constraint
    -l, --limit <FLOAT>       Set rapl power limit in watts
    -w, --window <INT>        Set rapl power window in microseconds
    -q, --quiet               Do not print table
    -h, --help                Print help information
    -V, --version             Print version information
```
```
$ krapl
 RAPL  Zone name  Long lim  Short lim  Long win     Short win  Usage
 ----  ---------  --------  ---------  -----------  ---------  ------
 0     package-0  4 W       15 W       27983872 μs  2440 μs    659 mW
 0:0   core       0 W       •          976 μs       •          89 mW
 0:1   uncore     0 W       •          976 μs       •          2 mW
```

### ki915

View or set i915 values.

```
$ ki915 -h
ki915 0.6.0

USAGE:
    ki915 [OPTIONS]

OPTIONS:
    -c, --card <IDS>     Target i915 drm card indexes or bus ids
    -n, --min <INT>      Set i915 min freq in megahertz
    -x, --max <INT>      Set i915 max freq in megahertz
    -b, --boost <INT>    Set i915 boost freq in megahertz
    -q, --quiet          Do not print table
    -h, --help           Print help information
    -V, --version        Print version information
```
```
$ ki915
 DRM  Driver  Actual   Req'd    Min      Max      Boost       Min lim  Max lim
 ---  ------  -------  -------  -------  -------  ----------  -------  -------
 1    i915    350 MHz  350 MHz  350 MHz  900 MHz  1000.0 MHz  350 MHz  1.1 GHz
```

### knvml

View or set nvidia management library values.

```
$ knvml -h
knvml 0.6.0

USAGE:
    knvml [OPTIONS]

OPTIONS:
    -c, --card <IDS>       Target nvml drm card indexes or bus ids
    -n, --gpu-min <INT>    Set nvml min gpu freq in megahertz
    -x, --gpu-max <INT>    Set nvml max gpu freq in megahertz
    -r, --gpu-reset        Reset nvml gpu freq to default
    -P, --power <FLOAT>    Set nvml device power limit in watts
    -R, --power-reset      Reset nvml power limit to default
    -q, --quiet            Do not print tables
    -h, --help             Print help information
    -V, --version          Print version information
```
```
$ knvml
 DRM  Driver  GPU cur  GPU lim  Power cur  Power lim  Min lim  Max lim
 ---  ------  -------  -------  ---------  ---------  -------  -------
 0    nvidia  1.1 GHz  2.1 GHz  27.1 W     •          •        •
```