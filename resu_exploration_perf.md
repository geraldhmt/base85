# result of try

## case with string

* String and no unsafe code
* only encode is rewrite
* 2 different implementation for noalloc and string
* * : only code
* * :
    * 2 codes to write for encode
    * slow code ?

* encode to string  : +33% of time vs u8 only

     Running benches/encode.rs (target/release/deps/encode-f386c10612224c6f)
Gnuplot not found, using plotters backend
encoder                 time:   [660.46 µs 666.72 µs 676.01 µs]
Found 22 outliers among 100 measurements (22.00%)
  15 (15.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe

encoder_prime           time:   [64.175 µs 64.206 µs 64.244 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

encoder_short           time:   [15.861 ns 15.908 ns 15.960 ns]
Found 23 outliers among 100 measurements (23.00%)
  18 (18.00%) low mild
  5 (5.00%) high severe

decoder                 time:   [301.75 µs 302.91 µs 304.38 µs]
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) high mild
  12 (12.00%) high severe

     Running benches/encode_noalloc.rs (target/release/deps/encode_noalloc-44a7f61b6eb8dd18)
Gnuplot not found, using plotters backend
encoder_noalloc         time:   [500.79 µs 501.54 µs 502.38 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

encoder_noalloc_prime   time:   [47.761 µs 47.800 µs 47.849 µs]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) high mild
  8 (8.00%) high severe

encoder_noalloc_short   time:   [8.9266 ns 8.9446 ns 8.9656 ns]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

decoder_noalloc         time:   [301.13 µs 302.21 µs 304.45 µs]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
