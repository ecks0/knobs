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

Utilities and subcommands accept `-h` for short help and `--help` for long help.

## Utility symlinks

### Installing symlinks

Alongside the `knobs` binary:
```
knobs install
```
To a particular directory:
```
knobs install /path/to/directory
```

### Uninstalling symlink

Alongside the `knobs` binary:
```
knobs install -u
```
From a particular directory:
```
knobs install -u /path/to/directory
```

## Argument groups

The cli accepts multiple argument groups, delimited by `--`. Useful
properties of argument groups:

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

## Environment variables

- `KNOBS_LOG` - Set to `trace` to see what's happening under the hood. Default `error`.
- `KNOBS_RAPL_SAMPLE_MS` - Set to a value between `1` (least accurate) and `1000` (most accurate) to control rapl `energy_uj` sample interval. Default `200`.
