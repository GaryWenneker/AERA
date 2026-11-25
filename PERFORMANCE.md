# Performance targets

Aspirational targets, not measured claims. Native reference hardware with SSD, same display and workload across comparisons where feasible. Separate hosted overhead from guest/native results.

| Metric | Target | Method |
|---|---|---|
| Cold boot to interactive desktop | <5 s | firmware excluded, external timestamp, 30 runs |
| Idle desktop RAM | <500 MiB | resident private/shared accounting documented, 5 min idle |
| Base installed image | <2 GiB | allocated bytes, no user data |
| Input-to-present latency | p95 <10 ms where hardware permits | instrumented input and display at 120 Hz or higher |
| Frame delivery | 120 Hz capable | p95 frame time <=8.33 ms on reference display |
| Idle CPU | <1% of one logical CPU | 5 minute sample, no background workloads |

Record machine, OS, compiler, commit, power mode, temperature, display, median/p95/p99 and raw data. Compare equivalent workloads against macOS/Windows/Linux; never claim universal speed superiority. P0 baseline: executable size, startup latency and peak RSS; no target verified yet. Add regression budgets after baseline and fail CI on reproducible regression.
