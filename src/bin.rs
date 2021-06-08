//
extern crate afrs;
use afrs::Rule;
use std::{
    fs,
    io::{prelude::*, BufReader},
    path::PathBuf,
};
use structopt::StructOpt;
//

#[derive(StructOpt)]
#[structopt(
    name = "afrs-cli",
    about = "Runs one or more AFRS rule over one or more files, returns line of file matching conditionals. Currently this only works for JSON objects, support for additional formats WIP."
)]
struct Opt {
    /// Provide one or more paths to valid JSON files. No validation is done on each object in a file. NDJson only for the time being.
    #[structopt(short, long, parse(from_os_str))]
    input: Vec<PathBuf>,

    /// Provide one or more paths to valid AFRS rule files. Deserializes using serde so can accept most formats, JSON for now.
    #[structopt(short, long, parse(from_os_str))]
    rules: Vec<PathBuf>,

    /// File to save output to. If this flag is not used output is sent to stdout.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

fn main() {
    let mut opt = Opt::from_args();
    // Build rules
    let mut rules: Vec<Rule> = Vec::new();
    while let Some(path) = opt.rules.pop() {
        let f = fs::File::open(path).unwrap();
        let r = BufReader::new(f);
        let rule: Rule = serde_json::from_reader(r).unwrap();
        rules.push(rule.validate().unwrap());
    }
    //
    if rules.len() == 0 {
        eprintln!("No valid rules provided.");
        std::process::exit(1);
    }
    // Output data
    while let Some(path) = opt.input.pop() {
        //
        let f = fs::File::open(path).unwrap();
        for line in BufReader::new(f).lines() {
            if let Ok(json_line) = line {
                for rule in &rules {
                    if let Ok(true) = rule.match_json(&json_line) {
                        println!("{:?}", rule.get_matches_json(&json_line));
                    }
                }
                // Output conditional
                match &opt.output {
                    Some(o) => {
                        let mut f = fs::File::open(o).unwrap();
                        let _ = f.write(json_line.as_bytes());
                        let _ = f.write(b"\n");
                    }
                    None => {}
                }
            }
        }
    }
}
