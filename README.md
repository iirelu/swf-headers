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

## FAQ

### Q: Why make a library for parsing swf files?

A: Why not? I had some experience with it, and I noticed there were no swf parsing tools on crates.io, so I identified my niche and ran with it.

### Q: Does this *really* need to rely on two decompression libraries just to parse a header?

A: Sadly, yes. The swf spec is awful, so swf files usually end up with half the header compressed with either zlib or LZMA.

### Q: Where's the swf spec?

A: [Here](https://www.adobe.com/content/dam/Adobe/en/devnet/swf/pdf/swf-file-format-spec.pdf). You'll probably want to read that through if you're planning on parsing the rest of the swf, but for understanding this library you just need the first chapter and page 27.

### Q: Hey, I have some public domain swf files using weird flash settings, do you want them?

A: Sure! Test coverage is proving to be the hardest part of this whole ordeal, so I could always welcome small swf files to test with.

### Q: Your code sucks. Can I fix it?

A: Also sure! Pull Requests are welcome.
