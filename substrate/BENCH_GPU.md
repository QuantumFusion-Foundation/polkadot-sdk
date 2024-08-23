# Bench GPU

## About
Each test is run 1_000_000 times division operation than 1_000_000 times add operation on the CPU and GPU separately.
Each test is running into self WASM VM instance.

## Running the bench
```bash
cd substrate
cargo bench -p sc-executor --bench bench_gpu
```

## Results
```bash
cd ./target/criterion/
```
Then open any ```report/index.html``` in any of the benches reports folder for each. Or open ```report/index.html``` in the ```target/criterion/report``` folder for all benches.
