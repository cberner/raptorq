# raptorq
Rust implementation of RaptorQ (RFC6330)

Recovery properties:
Reconstruction probability after receiving K + h packets = 1 - 1/256^(h + 1).
See "RaptorQ Technical Overview" by Qualcomm
