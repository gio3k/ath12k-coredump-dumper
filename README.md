# gio's ath12k coredump dumper

Cleaned up this tool a bit and added some CLI args so other people can use it

```
# from a coredump...
./dumper --input coredump.bin --output output/

# from a coredump, directly from cargo...
cargo run -- --input coredump.bin --output output/

# from an RDDM dump...
./dumper --input-type rddm --input coredump-rddm.bin --output output/
```

## Output example

The output files should look similar to this:

```
coredump-PagingData-6815952.bin
coredump-RddmData-7864560.bin
coredump-RemoteMemData-7405568.bin
rddm-AUX_DBG_DUMP.bin-8192.bin
rddm-AUX-M3.3.bin-49152.bin
rddm-ETB_Q6.bin-2048.bin
rddm-ETB_SOC.bin-16384.bin
rddm-ETB_WCSS.bin-65536.bin
rddm-PHYA_DBG_DUMP.bin-32768.bin
rddm-PHYA-M3.3.bin-278528.bin
rddm-PHYB_DBG_DUMP.bin-32768.bin
rddm-PHYB-M3.3.bin-278528.bin
rddm-Q6-SFR.bin-80.bin
rddm-Q6-SRAM.bin-4620288.bin
rddm-WLAON_DUMP.bin-3180.bin
```

## Notes

rddm: Q6-SFR seems to be the error message (SFR == "small fail reason"?)
