---
layout: default
title: Benchmarks
nav_order: 7
permalink: /benchmarks
---

# Benchmarks

The Snowbridge Polkadot runtime is benchmarked on the hardware and software described here. A deployment of the runtime must match or exceed this baseline in order to be secure.

Various other base assumptions are made in order to benchmark the runtime. These are documented more generally for Substrate & Polkadot [here](https://www.shawntabrizi.com/substrate-graph-benchmarks/docs/#/assumptions).

## Hardware

We used an **c5a.2xlarge** AWS EC2 instance with an SSD-backed EBS volume. The SSD is general purpose (gp2).

### Processor

<details>
<summary>lscpu - AMD EPYC 7R32 @ 2.2GHz (expand)</summary>

<pre>
Architecture:                    x86_64
CPU op-mode(s):                  32-bit, 64-bit
Byte Order:                      Little Endian
Address sizes:                   48 bits physical, 48 bits virtual
CPU(s):                          8
On-line CPU(s) list:             0-7
Thread(s) per core:              2
Core(s) per socket:              4
Socket(s):                       1
NUMA node(s):                    1
Vendor ID:                       AuthenticAMD
CPU family:                      23
Model:                           49
Model name:                      AMD EPYC 7R32
Stepping:                        0
CPU MHz:                         2161.934
BogoMIPS:                        5599.98
Hypervisor vendor:               KVM
Virtualization type:             full
L1d cache:                       128 KiB
L1i cache:                       128 KiB
L2 cache:                        2 MiB
L3 cache:                        16 MiB
NUMA node0 CPU(s):               0-7
Vulnerability Itlb multihit:     Not affected
Vulnerability L1tf:              Not affected
Vulnerability Mds:               Not affected
Vulnerability Meltdown:          Not affected
Vulnerability Spec store bypass: Mitigation; Speculative Store Bypass disabled via prctl and seccomp
Vulnerability Spectre v1:        Mitigation; usercopy/swapgs barriers and __user pointer sanitization
Vulnerability Spectre v2:        Mitigation; Full AMD retpoline, IBPB conditional, IBRS_FW, STIBP con
                                 ditional, RSB filling
Vulnerability Srbds:             Not affected
Vulnerability Tsx async abort:   Not affected
Flags:                           fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat ps
                                 e36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb 
                                 rdtscp lm constant_tsc rep_good nopl nonstop_tsc cpuid extd_apicid a
                                 perfmperf tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 
                                 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy
                                  cr8_legacy abm sse4a misalignsse 3dnowprefetch topoext ssbd ibrs ib
                                 pb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 rdseed adx smap clflus
                                 hopt clwb sha_ni xsaveopt xsavec xgetbv1 clzero xsaveerptr wbnoinvd 
                                 arat npt nrip_save rdpid
</pre>
</details>

### Memory

16GB instance type - clockspeed unknown.

### Hard Drive

Base disk speed is benchmarked with the following command:
```bash
fio --randrepeat=1 --ioengine=posixaio --direct=1 --gtod_reduce=1 --name=test --filename=test --bs=4k --iodepth=64 --size=4G --readwrite=randrw --rwmixread=75
```

<details>
<summary>fio disk benchmark (expand)</summary>

<pre>
test: (g=0): rw=randrw, bs=(R) 4096B-4096B, (W) 4096B-4096B, (T) 4096B-4096B, ioengine=posixaio, iodepth=64
fio-3.16
Starting 1 process
test: Laying out IO file (1 file / 4096MiB)
Jobs: 1 (f=1): [m(1)][100.0%][r=7735KiB/s,w=2514KiB/s][r=1933,w=628 IOPS][eta 00m:00s]
test: (groupid=0, jobs=1): err= 0: pid=234088: Thu May 13 23:40:01 2021
  read: IOPS=1927, BW=7709KiB/s (7894kB/s)(3070MiB/407787msec)
   bw (  KiB/s): min= 6456, max=11752, per=99.98%, avg=7707.10, stdev=585.05, samples=815
   iops        : min= 1614, max= 2938, avg=1926.76, stdev=146.26, samples=815
  write: IOPS=644, BW=2576KiB/s (2638kB/s)(1026MiB/407787msec); 0 zone resets
   bw (  KiB/s): min= 2000, max= 3800, per=99.99%, avg=2575.80, stdev=223.94, samples=815
   iops        : min=  500, max=  950, avg=643.90, stdev=56.00, samples=815
  cpu          : usr=1.04%, sys=0.16%, ctx=131126, majf=0, minf=42
  IO depths    : 1=0.1%, 2=0.1%, 4=0.1%, 8=12.5%, 16=25.0%, 32=50.0%, >=64=12.5%
     submit    : 0=0.0%, 4=100.0%, 8=0.0%, 16=0.0%, 32=0.0%, 64=0.0%, >=64=0.0%
     complete  : 0=0.0%, 4=98.6%, 8=0.1%, 16=0.0%, 32=0.0%, 64=1.4%, >=64=0.0%
     issued rwts: total=785920,262656,0,0 short=0,0,0,0 dropped=0,0,0,0
     latency   : target=0, window=0, percentile=100.00%, depth=64

Run status group 0 (all jobs):
   READ: bw=7709KiB/s (7894kB/s), 7709KiB/s-7709KiB/s (7894kB/s-7894kB/s), io=3070MiB (3219MB), run=407787-407787msec
  WRITE: bw=2576KiB/s (2638kB/s), 2576KiB/s-2576KiB/s (2638kB/s-2638kB/s), io=1026MiB (1076MB), run=407787-407787msec

Disk stats (read/write):
  nvme0n1: ios=785829/262660, merge=0/15, ticks=270784/126336, in_queue=100, util=100.00%
</pre>
</details>

### Drivers

<details>
<summary>lspci driver details (expand)</summary>

<pre>
00:00.0 Host bridge: Intel Corporation 440FX - 82441FX PMC [Natoma]
00:01.0 ISA bridge: Intel Corporation 82371SB PIIX3 ISA [Natoma/Triton II]
00:01.3 Non-VGA unclassified device: Intel Corporation 82371AB/EB/MB PIIX4 ACPI (rev 08)
00:03.0 VGA compatible controller: Amazon.com, Inc. Device 1111
00:04.0 Non-Volatile memory controller: Amazon.com, Inc. Device 8061
00:05.0 Ethernet controller: Amazon.com, Inc. Elastic Network Adapter (ENA)
</pre>
</details>

### Other Hardware

<details>
<summary>lshw (expand)</summary>

<pre>
    description: Computer
    product: c5a.2xlarge
    vendor: Amazon EC2
    serial: ec25f941-cb6e-442e-0699-8e8e393daba2
    width: 64 bits
    capabilities: smbios-2.7 dmi-2.7 smp vsyscall32
    configuration: uuid=41F925EC-6ECB-2E44-0699-8E8E393DABA2
  *-core
       description: Motherboard
       vendor: Amazon EC2
       physical id: 0
     *-firmware
          description: BIOS
          vendor: Amazon EC2
          physical id: 0
          version: 1.0
          date: 10/16/2017
          size: 64KiB
          capacity: 64KiB
          capabilities: pci edd acpi virtualmachine
     *-cpu
          description: CPU
          product: AMD EPYC 7R32
          vendor: Advanced Micro Devices [AMD]
          physical id: 4
          bus info: cpu@0
          version: AMD EPYC 7R32
          slot: CPU 0
          size: 2800MHz
          capacity: 3300MHz
          width: 64 bits
          clock: 100MHz
          capabilities: lm fpu fpu_exception wp vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp x86-64 constant_tsc rep_good nopl nonstop_tsc cpuid extd_apicid aperfmperf tsc_known_freq pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor lahf_lm cmp_legacy cr8_legacy abm sse4a misalignsse 3dnowprefetch topoext ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2 rdseed adx smap clflushopt clwb sha_ni xsaveopt xsavec xgetbv1 clzero xsaveerptr wbnoinvd arat npt nrip_save rdpid
          configuration: cores=4 enabledcores=4 threads=8
     *-memory
          description: System memory
          physical id: 1
          size: 15GiB
     *-pci
          description: Host bridge
          product: 440FX - 82441FX PMC [Natoma]
          vendor: Intel Corporation
          physical id: 100
          bus info: pci@0000:00:00.0
          version: 00
          width: 32 bits
          clock: 33MHz
        *-isa
             description: ISA bridge
             product: 82371SB PIIX3 ISA [Natoma/Triton II]
             vendor: Intel Corporation
             physical id: 1
             bus info: pci@0000:00:01.0
             version: 00
             width: 32 bits
             clock: 33MHz
             capabilities: isa
             configuration: latency=0
        *-generic UNCLAIMED
             description: Non-VGA unclassified device
             product: 82371AB/EB/MB PIIX4 ACPI
             vendor: Intel Corporation
             physical id: 1.3
             bus info: pci@0000:00:01.3
             version: 08
             width: 32 bits
             clock: 33MHz
             configuration: latency=0
        *-display UNCLAIMED
             description: VGA compatible controller
             product: Amazon.com, Inc.
             vendor: Amazon.com, Inc.
             physical id: 3
             bus info: pci@0000:00:03.0
             version: 00
             width: 32 bits
             clock: 33MHz
             capabilities: vga_controller
             configuration: latency=0
             resources: memory:fe400000-fe7fffff memory:c0000-dffff
        *-storage
             description: Non-Volatile memory controller
             product: Amazon.com, Inc.
             vendor: Amazon.com, Inc.
             physical id: 4
             bus info: pci@0000:00:04.0
             version: 00
             width: 32 bits
             clock: 33MHz
             capabilities: storage pciexpress msix nvm_express bus_master cap_list
             configuration: driver=nvme latency=0
             resources: irq:11 memory:febf0000-febf3fff
           *-nvme0
                description: NVMe device
                product: Amazon Elastic Block Store
                physical id: 0
                logical name: /dev/nvme0
                version: 1.0
                serial: vol0e2cdf703888f46f6
                configuration: nqn=nqn.2014.08.org.nvmexpress:1d0f1d0fvol0e2cdf703888f46f6Amazon Elastic Block Store state=live
              *-namespace
                   description: NVMe namespace
                   physical id: 1
                   logical name: /dev/nvme0n1
                   size: 100GiB (107GB)
                   capabilities: partitioned partitioned:dos
                   configuration: logicalsectorsize=512 sectorsize=512 signature=5198cbc0
                 *-volume
                      description: EXT4 volume
                      vendor: Linux
                      physical id: 1
                      logical name: /dev/nvme0n1p1
                      logical name: /
                      version: 1.0
                      serial: e8070c31-bfee-4314-a151-d1332dc23486
                      size: 99GiB
                      capacity: 99GiB
                      capabilities: primary bootable journaled extended_attributes large_files huge_files dir_nlink recover 64bit extents ext4 ext2 initialized
                      configuration: created=2021-04-30 23:30:56 filesystem=ext4 label=cloudimg-rootfs lastmountpoint=/ modified=2021-04-30 23:40:03 mount.fstype=ext4 mount.options=rw,relatime,discard mounted=2021-05-07 03:13:05 state=mounted
        *-network
             description: Ethernet interface
             product: Elastic Network Adapter (ENA)
             vendor: Amazon.com, Inc.
             physical id: 5
             bus info: pci@0000:00:05.0
             logical name: ens5
             version: 00
             serial: 0a:d0:04:bc:4f:8f
             width: 32 bits
             clock: 33MHz
             capabilities: pciexpress msix bus_master cap_list ethernet physical
             configuration: broadcast=yes driver=ena ip=172.31.45.174 latency=0 link=yes multicast=yes
             resources: irq:0 memory:febf4000-febf7fff memory:fe800000-fe8fffff
     *-pnp00:00
          product: PnP device PNP0b00
          physical id: 2
          capabilities: pnp
          configuration: driver=rtc_cmos
     *-pnp00:01
          product: PnP device PNP0303
          physical id: 3
          capabilities: pnp
          configuration: driver=i8042 kbd
     *-pnp00:02
          product: PnP device PNP0f13
          physical id: 5
          capabilities: pnp
          configuration: driver=i8042 aux
     *-pnp00:03
          product: PnP device PNP0400
          physical id: 6
          capabilities: pnp
     *-pnp00:04
          product: PnP device PNP0501
          physical id: 7
          capabilities: pnp
          configuration: driver=serial
</pre>
</details>

## Software

### OS

<details>
<summary>Ubuntu 20.04 (expand)</summary>

<pre>
$  lsb_release -a
No LSB modules are available.
Distributor ID:	Ubuntu
Description:	Ubuntu 20.04.2 LTS
Release:	20.04
Codename:	focal
</pre>
</details>

### Rust

<details>
<summary>cargo version (expand)</summary>

<pre>
$ cargo --version
cargo 1.52.0 (69767412a 2021-04-21)
$ rustc --version
rustc 1.52.0 (88f19c6da 2021-05-03)
</pre>
</details>
