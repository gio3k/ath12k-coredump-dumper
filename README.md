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

## Notes
coredump: RemoteMemData is 

rddm: Q6-SFR seems to be the error message (SFR == "small fail reason"?)





