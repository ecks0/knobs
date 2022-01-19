# knobs

Command-line utilities for controlling Linux performance settings.

Utilities may be executed as subcommands of the `knobs` binary, or as
stand-alone utilities a la `busybox`.

To install symlinks for `knobs` utilities alongside the `knobs` binary,
run:
```
knobs install
```

To install symlinks to a particular directory, run:
```
knobs install /path/to/directory
```

## kcpu

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

## krapl

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

## k915

```
$ k915 -h
k915 0.6.0

USAGE:
    k915 [OPTIONS]

OPTIONS:
    -c, --card <IDS>     Target i915 drm card indexes or bus ids
    -n, --min <INT>      Set i915 min freq in megahertz
    -x, --max <INT>      Set i915 max freq in megahertz
    -b, --boost <INT>    Set i915 boost freq in megahertz
    -q, --quiet          Do not print table
    -h, --help           Print help information
    -V, --version        Print version information
```

## knvml

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