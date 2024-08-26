# dupimg

A minimal command-line duplicate image finder with persistent caching.

## Summary

Checks the similarity of images specified on the command line, by hashing them
and computing their hamming distance using the
[image_hasher](https://github.com/qarmin/img_hash) library. Likely duplicates
are printed in groups on the command line.

Both hash and hamming distance computations are multithreaded; comparing 2762
images takes ~25 seconds on a Ryzen 9 5900X.

Computed image hashes are persistently stored in CSV files under
`~/.cache/dupimg/`, so that only the hamming distance calculations has to be
re-done on subsequent comparisons with the same image.

## Installation

Either run `cargo install dupimg`, or clone this repository and run
`cargo install --path .` from the project root.

## Basic usage

`dupimg [-r directory1/ directory2/ ...] file1.jpg file2.png ...`

For additional information, see `dupimg --help`.

### Output format

Output groups are sorted alphabetically by path.

```
<PATH 1>            # first image
<DIST>  <PATH 2>    # hamming distance, first likely duplicate
<DIST>  [PATH ...]  # other likely duplicates

<PATH 3>            # second image
<DIST>  <PATH 4>
<DIST>  [PATH ...]

[...]
```

## Recurse

`-r` may be specified to enable traversing specified directories.

When recurse is enabled, only PNG and JPG files will be checked. This also
applies to filenames specified on the command line.

## "Left-right" comparison

`-l <FILE/DIRECTORY>` may be specified to perform comparisons between two
distinct sets of images -- aka. determine which images in the "left" set are
also present in the "right" set, instead of comparing all images with each
other.

`-l` must be specified per file/directory in order to assign them to the "left"
set. It works in combination with `-r`/`--recurse`: e.g. `-r -l dir1/ dir2/`
compares all images under `dir1/` with all images under `dir2/`.

When `-l` is specified for a single file only, dupimg effectively becomes a
local reverse image search utility.

## Threshold

`-t <THRESHOLD>`, where THRESHOLD is a positive integer, may be specified to
adjust the duplicate detection threshold.

The default is 5, which with the default hash size errs on the side of caution,
somewhat preferring false positives over false negatives. 0 gives very few false
positives, but might miss some duplicates (e.g. due to compression artifacts).

## Hash size

`-h <SIZE>`, where `SIZE` is a positive integer, may be specified to change the
size of image hashes.

Different hash sizes are not comparable and are thus stored separately
under `~/.cache/dupimg/`.

The default hash size is 8 bytes, which works reasonably well for most images.
Note that the detection threshold must be increased together with the hash size.

## Credits

All code in this crate was written by myself.

All credits for libraries used go to their respective authors.
