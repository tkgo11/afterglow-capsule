# Public historical reference vector

`reference-historical.tle` is the public armored ciphertext from
`github.com/drand/tlock`, commit `3ea7fbb59e85d00b0d9b6b2554e9652d5766811b`,
`tlock_test.go`, `TestDecryptText`. Its expected public plaintext is `hello world`.
The upstream repository supplies MIT and Apache-2.0 license files.

Network parameters and the already-public historical beacon are centralized in
`docs/external-assumptions.md`; the spike script reads that fixture. This vector
provides historical Go → Rust compatibility evidence. It does not replace freshly
generated bidirectional, live future-round or Windows tests required by Spike A.
