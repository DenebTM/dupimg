# dupimg

A simple duplicate image checker.

## Summary

Checks the similarity between two images, using the SSIM algorithm implemented
by `dssim-core`. Works on JPG and PNG images of any size by first rescaling
them to 200x200 internally.

Runs multithreaded, every image is checked against every other one and there
is no heuristic for e.g. finding likely candidates early, so while this is
somewhat optimized, checking many files will likely still be very slow.

Note: I am rather inexperienced with Rust, so there will definitely be some
dumb code here.

## Installation
Clone this repository and run `cargo install --path .`

## Basic usage

`dupimg [-r directory1/ directory2/ ...] file1.jpg file2.png ...`

### Output format

```
<IMAGE 1>       # path to first image
<IMAGE 2>       # path to second image
  SSIM: 0.0...  # positive and unbounded; lower values indicate a closer match
```

### Threshold

The threshold for displaying a match may be set by e.g. `-t 0.01`. Only matches
with a SSIM smaller or equal to this threshold will be displayed.

The default is 0.1, as this tends to give good results with very few false
positives.

### Recurse

`-r` may be specified to enable traversing specified directories.

When recurse is enabled, only PNG and JPG files will be checked. This also
applies to filenames specified on the command line.

### Note

All program output is unsorted and unstable, do not rely on it.

## Credits

All code in this crate was written by myself.

All credits for libraries used go to their respective authors.
