# rust-cloc
Count lines from files in a directory.

## Features
- Count the number of empty and non-empty lines in total from all files in a directory.
- Count the number of empty and non-empty lines for each file type in a directory.
- Multithreading with `rayon` to count the number of lines for separate files in parallel.
- Uses `clap` for command-line argument parsing.

## Usage
For example, running
```
cargo run -- src
```
in this repository prints the following:
```
There are 172 lines of code.
There are 21 empty lines.
10.88% of the lines are empty.
```

Running
```
cargo run -- -A src
```
in this repository prints:
```
There are 172 lines of code in "rs" files.
There are 21 empty lines in "rs" files.
10.88% of the lines in "rs" files are empty.
```
