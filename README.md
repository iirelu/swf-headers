# swf-headers

A library for reading the headers of a swf file, and optionally for helping you read the rest of it, too.

## Example:

```rust
extern crate swf_headers;

use std::io::Read; // Needed for calling read_to_end()

use swf_headers::SwfHeaders;
use swf_headers::Error as SwfError;
use swf_headers::DecodedSwf;

let (headers, mut decoded_swf) = SwfHeaders::open("example.swf").unwrap_or_else(|err| {
    match err {
        SwfError::IoError(_) => panic!("Oh no! An IO error!"),
        SwfError::NotSwf => panic!("Oh no! It wasn't actually a swf file!")
    }
});

println!("The compression method is {:?}", headers.signature());
println!("The swf version is {}", headers.version());
println!("The file length in bytes is {}", headers.file_length());
println!("The dimensions in pixels are {:?}", headers.dimensions());
println!("The frame rate is {}", headers.frame_rate());
println!("And finally, the frame count is {}!", headers.frame_count());

let mut the_rest_of_the_swf: Vec<u8> = vec![];
decoded_swf.read_to_end(&mut the_rest_of_the_swf).ok().expect("Oh no! Error reading!");
// And then you can do whatever you want with the rest of the swf!
```

## Testing

Testing is a pain when you have to test on proprietary blobs. See tests/README.md for more information.
