# tokio-process-pipes
Exploring tokio-process async pipes io

This example shows how to read child processes stdout as streams of lines
in an async fashion.

Building and running will produce something like this:

    $ cargo build
    $ ./target/debug/tokio-process-pipes

    LINE: [CHILD 88187] PING 127.0.0.1 (127.0.0.1): 56 data bytes
    LINE: [CHILD 88187] 64 bytes from 127.0.0.1: icmp_seq=0 ttl=64 time=0.123 ms
    LINE: [CHILD 88187] 64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.110 ms
    LINE: [CHILD 88188] PING 0.0.0.0 (0.0.0.0): 56 data bytes
    LINE: [CHILD 88188] Request timeout for icmp_seq 0
    LINE: [CHILD 88188] Request timeout for icmp_seq 1
    LINE: [CHILD 88187] 64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.113 ms


I mostly built this to explore async reading of child stdout so I can run
such child processes from single-threaded Web services. That is: I want to
serve Web requets while reading child stdout.


