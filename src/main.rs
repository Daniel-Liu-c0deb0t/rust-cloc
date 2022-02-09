use clap::Parser;

use rayon::prelude::*;

use std::fs::{self, *};
use std::path::*;
use std::io::*;
use std::collections::HashMap;

fn main() {
    let args = Args::parse();

    let mut files = Vec::new();
    find_all_files(Path::new(&args.directory), &mut files);

    if args.threads > 1 {
        rayon::ThreadPoolBuilder::new().num_threads(args.threads).build_global().unwrap();
    }

    if args.by_ext {
        let res_map = count_lines_by_ext(&files, args.threads);
        res_map.iter().for_each(|(ext, res)| {
            println!("There are {} lines of code in \"{}\" files.", res.lines_of_code, ext);
            println!("There are {} empty lines in \"{}\" files.", res.empty_lines, ext);
            println!("{:.2}% of the lines in \"{}\" files are empty.", res.percent_empty(), ext);
        });
    } else {
        let res = count_lines(&files, args.threads);
        println!("There are {} lines of code.", res.lines_of_code);
        println!("There are {} empty lines.", res.empty_lines);
        println!("{:.2}% of the lines are empty.", res.percent_empty());
    }
}

/// Count the number of empty and non-empty lines in a file.
fn count_lines_in_file(path: &PathBuf) -> Results {
    let r = BufReader::new(File::open(path).expect("Unable to open file"));
    let mut res = Results::new();

    for line in r.lines() {
        let line = match line {
            Ok(l) => l,
            _ => return Results::new(),
        };

        if line.trim().is_empty() {
            res.empty_lines += 1;
        } else {
            res.lines_of_code += 1;
        }
    }

    res
}

/// Count lines for multiple files specified by their paths, and aggregate
/// the line counts across all files.
fn count_lines(files: &[PathBuf], threads: usize) -> Results {
    let reduce_fn = |a: Results, b: Results| Results {
        lines_of_code: a.lines_of_code + b.lines_of_code,
        empty_lines: a.empty_lines + b.empty_lines,
    };

    if threads > 1 {
        files.par_iter().map(count_lines_in_file).reduce(
            || Results::new(),
            reduce_fn
        )
    } else {
        files.iter().map(count_lines_in_file).fold(
            Results::new(),
            reduce_fn
        )
    }
}

/// Count lines and aggregate the counts for each file type.
fn count_lines_by_ext(files: &[PathBuf], threads: usize) -> HashMap<String, Results> {
    let reduce_fn = |mut map: HashMap<String, Results>, (new_ext, new_res): (String, Results)| {
        map.entry(new_ext)
            .and_modify(|e| {
                e.lines_of_code += new_res.lines_of_code;
                e.empty_lines += new_res.empty_lines;
            })
            .or_insert(new_res);
        map
    };

    if threads > 1 {
        files.par_iter().map(|p| (get_ext(p), count_lines_in_file(p))).fold(
            || HashMap::new(),
            reduce_fn
        ).reduce(|| { HashMap::new() }, |mut a, b| {
            for e in b.into_iter() {
                a = reduce_fn(a, e);
            }
            a
        })
    } else {
        files.iter().map(|p| (get_ext(p), count_lines_in_file(p))).fold(
            HashMap::new(),
            reduce_fn
        )
    }
}

/// Recursively explore a directory to get a list of file paths.
fn find_all_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read path") {
            let entry = entry.unwrap();
            let curr_path = entry.path();

            if curr_path.is_dir() {
                find_all_files(&curr_path, files);
            } else {
                files.push(curr_path);
            }
        }
    }
}

/// Get the extension of a path.
fn get_ext(path: &Path) -> String {
    match path.extension() {
        Some(p) => p.to_str().unwrap_or("").to_owned(),
        None => "".to_owned(),
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short = 'A', long)]
    by_ext: bool,
    directory: String,
    #[clap(short = 'j', long, default_value_t = 1)]
    threads: usize,
}

/// Holds the number of empty and non-empty lines in a file.
struct Results {
    lines_of_code: usize,
    empty_lines: usize,
}

impl Results {
    /// Create a new zero-initialized Results instance.
    fn new() -> Self {
        Results { lines_of_code: 0, empty_lines: 0 }
    }

    /// Compute the percentage of empty lines.
    fn percent_empty(&self) -> f64 {
        (self.empty_lines as f64) / ((self.lines_of_code + self.empty_lines) as f64)
    }
}
