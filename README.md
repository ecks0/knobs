# knobs

## About

Display and set system tunables.

## Supported tunables

- cpu
  - online status
- cpufreq
  - governor
  - min/max frequencies
- intel_pstate
  - epb
  - epp
- drm
  - i915
    - min/max/boost frequencies
- nvml (nvidia management library)
  - graphics clock min/max frequency
  - power limit

## Example usage

---

### Set cpufreq min/max frequency

**For all CPUs**

```
knobs -n 800mhz -x 4.4ghz
```
...or with long args...
```
knobs --cpufreq-min 800mhz --cpufreq-max 4.4ghz
```

**For the first 4 CPUs only**

```
knobs -c 0-3 -n 800mz -x 4.4ghz
```
...or with long args...
```
knobs --cpu 0-3 --cpufreq-min 800mhz --cpufreq-max 4.4ghz
```

---

### Set intel_pstate epb

**For all CPUs**

```
knobs --pstate-epb 6
```

**For the first 4 CPUs only**

```
knobs -c 0-3 --pstate-epb 6
```
...or with long args...
```
knobs --cpu 0-3 --pstate-epb 6
```

---

### Set nvidia min/max GPU frequency

**For all GPUs**

```
knobs --nvml-gpu-clock 600mhz,2.2ghz
```

**For the first 2 GPUs only**

```
knobs --nvml 0,1 --nvml-gpu-clock 600mhz,2.2ghz
```

## Example Output

```
 CPU  Online  Governor   Cur      Min      Max      CPU min  CPU max
 ---  ------  --------   ---      ---      ---      -------  -------
 0    •       schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 1    true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 2    true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 3    true    schedutil  4.3 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 4    true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 5    true    schedutil  4.3 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 6    true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 7    true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 8    true    schedutil  4.1 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 9    true    schedutil  4.1 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 10   true    schedutil  4.4 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz
 11   true    schedutil  4.2 GHz  800 MHz  4.4 GHz  800 MHz  5.0 GHz

 CPU  Available governors
 ---  -------------------
 all  conservative ondemand userspace powersave performance schedutil

 intel_pstate: passive

          Nvidia GPU  0
 -------------------  -------------------
                Name  GeForce RTX 2080 Ti
              PCI ID  00000000:02:00.0
    Graphics cur/max  690 MHz / 2.2 GHz
      Memory cur/max  810 MHz / 7.0 GHz
          SM cur/max  690 MHz / 2.2 GHz
       Video cur/max  630 MHz / 1.9 GHz
   Memory used/total  1.1 GB / 11.6 GB
    Power used/limit  23 W / 260 W
 Power limit min/max  100 W / 325 W
```

## Help

```
knobs 0.2.0

USAGE:
    knobs [OPTIONS]

OPTIONS:
        --show-cpu                     Print cpu and cpufreq values
        --show-pstate                  Print intel_pstate values
        --show-drm                     Print drm values
        --show-nvml                    Print nvidia management values
    -c, --cpu <INDICES>                Target cpu ids, default all, ex. 0,1,3-5
    -o, --cpu-on <0|1>                 Set cpu online status per --cpu
    -O, --cpu-on-each <[0|1|-]+>       Set cpu online status, ex. 10-1 → 0=ON 1=OFF 2=SKIP 3=ON
    -g, --cpufreq-gov <NAME>           Set cpufreq governor per --cpu
    -n, --cpufreq-min <HZ>             Set cpufreq min freq per --cpu, ex. 1200, 1200mhz, 1.2ghz
    -x, --cpufreq-max <HZ>             Set cpufreq max freq per --cpu, ex. 1200, 1200mhz, 1.2ghz
        --pstate-epb <0-15>            Set intel_pstate energy/performance bias per --cpu
        --pstate-epp <NAME>            Set intel_pstate energy/performance pref per --cpu
        --drm-i915 <INDICES>           Target i915 card ids, default all, ex. 0,1,3-5
        --drm-i915-min <HZ>            Set i915 min frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz
        --drm-i915-max <HZ>            Set i915 max frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz
        --drm-i915-boost <HZ>          Set i915 boost frequency per --drm-i915, ex. 1200, 1200mhz, 1.2ghz
        --nvml <INDICES>               Target nvidia gpu ids, default all, ex. 0,1,3-5
        --nvml-gpu-clock <HZ|HZ,HZ>    Set nvidia gpu min,max frequency per --nvml, ex. 1200mhz or 900mhz,1.4ghz
        --nvml-gpu-clock-reset         Reset nvidia gpu min,max frequency per --nvml
        --nvml-power-limit <WATTS>     Set nvidia gpu power limit per --nvml, ex. 260, 260w, 0.26kw
    -h, --help                         Prints help information

ENVS:
        KNOBS_LOG=<error|warn|info|debug|trace>    Log level, default error
```