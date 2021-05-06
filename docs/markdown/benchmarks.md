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

We used an **m5d.2xlarge** AWS EC2 instance with NVMe SSD.

### Processor

<details>
<summary>lscpu - Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz (expand)</summary>

<pre>
Architecture:                    x86_64
CPU op-mode(s):                  32-bit, 64-bit
Byte Order:                      Little Endian
Address sizes:                   46 bits physical, 48 bits virtual
CPU(s):                          8
On-line CPU(s) list:             0-7
Thread(s) per core:              2
Core(s) per socket:              4
Socket(s):                       1
NUMA node(s):                    1
Vendor ID:                       GenuineIntel
CPU family:                      6
Model:                           85
Model name:                      Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz
Stepping:                        7
CPU MHz:                         3099.089
BogoMIPS:                        4999.99
Hypervisor vendor:               KVM
Virtualization type:             full
L1d cache:                       128 KiB
L1i cache:                       128 KiB
L2 cache:                        4 MiB
L3 cache:                        35.8 MiB
NUMA node0 CPU(s):               0-7
Vulnerability Itlb multihit:     KVM: Vulnerable
Vulnerability L1tf:              Mitigation; PTE Inversion
Vulnerability Mds:               Vulnerable: Clear CPU buffers attempted, no microcode; SMT Host stat
                                 e unknown
Vulnerability Meltdown:          Mitigation; PTI
Vulnerability Spec store bypass: Vulnerable
Vulnerability Spectre v1:        Mitigation; usercopy/swapgs barriers and __user pointer sanitization
Vulnerability Spectre v2:        Mitigation; Full generic retpoline, STIBP disabled, RSB filling
Vulnerability Srbds:             Not affected
Vulnerability Tsx async abort:   Not affected
Flags:                           fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat ps
                                 e36 clflush mmx fxsr sse sse2 ss ht syscall nx pdpe1gb rdtscp lm con
                                 stant_tsc rep_good nopl xtopology nonstop_tsc cpuid aperfmperf tsc_k
                                 nown_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic mov
                                 be popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor la
                                 hf_lm abm 3dnowprefetch invpcid_single pti fsgsbase tsc_adjust bmi1 
                                 avx2 smep bmi2 erms invpcid mpx avx512f avx512dq rdseed adx smap clf
                                 lushopt clwb avx512cd avx512bw avx512vl xsaveopt xsavec xgetbv1 xsav
                                 es ida arat pku ospke
</pre>
</details>

### Memory

32GB instance type - clockspeed unknown.

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
Jobs: 1 (f=1): [m(1)][100.0%][r=6528KiB/s,w=2176KiB/s][r=1632,w=544 IOPS][eta 00m:00s]
test: (groupid=0, jobs=1): err= 0: pid=30830: Thu May  6 06:13:32 2021
  read: IOPS=1750, BW=7000KiB/s (7168kB/s)(3070MiB/449076msec)
   bw (  KiB/s): min= 3376, max= 9208, per=100.00%, avg=6999.69, stdev=694.95, samples=898
   iops        : min=  844, max= 2302, avg=1749.91, stdev=173.74, samples=898
  write: IOPS=584, BW=2340KiB/s (2396kB/s)(1026MiB/449076msec); 0 zone resets
   bw (  KiB/s): min= 1232, max= 3112, per=100.00%, avg=2339.39, stdev=250.83, samples=898
   iops        : min=  308, max=  778, avg=584.83, stdev=62.71, samples=898
  cpu          : usr=0.36%, sys=0.12%, ctx=131171, majf=0, minf=42
  IO depths    : 1=0.1%, 2=0.1%, 4=0.1%, 8=12.5%, 16=25.0%, 32=50.0%, >=64=12.5%
     submit    : 0=0.0%, 4=100.0%, 8=0.0%, 16=0.0%, 32=0.0%, 64=0.0%, >=64=0.0%
     complete  : 0=0.0%, 4=98.6%, 8=0.1%, 16=0.0%, 32=0.0%, 64=1.4%, >=64=0.0%
     issued rwts: total=785920,262656,0,0 short=0,0,0,0 dropped=0,0,0,0
     latency   : target=0, window=0, percentile=100.00%, depth=64

Run status group 0 (all jobs):
   READ: bw=7000KiB/s (7168kB/s), 7000KiB/s-7000KiB/s (7168kB/s-7168kB/s), io=3070MiB (3219MB), run=449076-449076msec
  WRITE: bw=2340KiB/s (2396kB/s), 2340KiB/s-2340KiB/s (2396kB/s-2396kB/s), io=1026MiB (1076MB), run=449076-449076msec

Disk stats (read/write):
  nvme0n1: ios=785535/262632, merge=0/56, ticks=286281/154058, in_queue=2516, util=100.00%
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
00:1f.0 Non-Volatile memory controller: Amazon.com, Inc. NVMe SSD Controller
</pre>
</details>

### Other Hardware

<details>
<summary>lshw (expand)</summary>

<pre>
    description: Computer
    product: m5d.2xlarge
    vendor: Amazon EC2
    serial: ec2c7c76-ee64-0e80-dc3c-c61709530182
    width: 64 bits
    capabilities: smbios-2.7 dmi-2.7 smp vsyscall32
    configuration: uuid=767C2CEC-64EE-800E-DC3C-C61709530182
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
          product: Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz
          vendor: Intel Corp.
          physical id: 4
          bus info: cpu@0
          version: Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz
          slot: CPU 0
          size: 2500MHz
          capacity: 3500MHz
          width: 64 bits
          clock: 100MHz
          capabilities: lm fpu fpu_exception wp vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ss ht syscall nx pdpe1gb rdtscp x86-64 constant_tsc rep_good nopl xtopology nonstop_tsc cpuid aperfmperf tsc_known_freq pni pclmulqdq ssse3 fma cx16 pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand hypervisor lahf_lm abm 3dnowprefetch invpcid_single pti fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx avx512f avx512dq rdseed adx smap clflushopt clwb avx512cd avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves ida arat pku ospke
          configuration: cores=4 enabledcores=4 threads=8
     *-memory
          description: System memory
          physical id: 1
          size: 31GiB
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
        *-storage:0
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
                serial: vol05e860e733ce11280
                configuration: nqn=nqn.2014.08.org.nvmexpress:1d0f1d0fvol05e860e733ce11280Amazon Elastic Block Store state=live
              *-namespace
                   description: NVMe namespace
                   physical id: 1
                   logical name: /dev/nvme0n1
                   size: 100GiB (107GB)
                   capabilities: partitioned partitioned:dos
                   configuration: logicalsectorsize=512 sectorsize=512 signature=8757600c
                 *-volume
                      description: EXT4 volume
                      vendor: Linux
                      physical id: 1
                      logical name: /dev/nvme0n1p1
                      logical name: /
                      version: 1.0
                      serial: 7969d789-20ae-4f61-84ff-c0ac50e0dd19
                      size: 99GiB
                      capacity: 99GiB
                      capabilities: primary bootable journaled extended_attributes large_files huge_files dir_nlink 64bit extents ext4 ext2 initialized
                      configuration: created=2021-02-23 23:47:58 filesystem=ext4 label=cloudimg-rootfs lastmountpoint=/ modified=2021-02-23 23:51:29 mount.fstype=ext4 mount.options=rw,relatime,discard mounted=2021-04-16 02:24:48 state=mounted
        *-network
             description: Ethernet interface
             product: Elastic Network Adapter (ENA)
             vendor: Amazon.com, Inc.
             physical id: 5
             bus info: pci@0000:00:05.0
             logical name: ens5
             version: 00
             serial: 0a:78:f1:4f:ec:1b
             width: 32 bits
             clock: 33MHz
             capabilities: pciexpress msix bus_master cap_list ethernet physical
             configuration: broadcast=yes driver=ena ip=172.31.44.1 latency=0 link=yes multicast=yes
             resources: irq:0 memory:febf4000-febf7fff memory:fe800000-fe8fffff memory:febe0000-febeffff
        *-storage:1
             description: Non-Volatile memory controller
             product: NVMe SSD Controller
             vendor: Amazon.com, Inc.
             physical id: 1f
             bus info: pci@0000:00:1f.0
             version: 00
             width: 32 bits
             clock: 33MHz
             capabilities: storage pciexpress msix nvm_express bus_master cap_list
             configuration: driver=nvme latency=0
             resources: irq:0 memory:febf8000-febfbfff memory:fe900000-fe901fff
           *-nvme1
                description: NVMe device
                product: Amazon EC2 NVMe Instance Storage
                physical id: 0
                logical name: /dev/nvme1
                version: 0
                serial: AWS27E94FE79AD555E6B
                configuration: nqn=nqn.2014.08.org.nvmexpress:1d0f0000AWS27E94FE79AD555E6BAmazon EC2 NVMe Instance Storage state=live
              *-namespace
                   description: NVMe namespace
                   physical id: 1
                   logical name: /dev/nvme1n1
                   size: 279GiB (300GB)
                   configuration: logicalsectorsize=512 sectorsize=512
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
cargo 1.51.0 (43b129a20 2021-03-16)
$ rustc --version
cargo 1.51.0 (43b129a20 2021-03-16)
</pre>
</details>
