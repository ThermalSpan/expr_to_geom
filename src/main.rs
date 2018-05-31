#[macro_use]
extern crate structopt;
extern crate implicit;
extern crate geoprim;
extern crate bincode;

use structopt::StructOpt;
use std::process::Command;
use implicit::function::Function;
use implicit::mtree::*;
use implicit::interval::Interval;
use std::collections::HashMap;
use geoprim::Plot;
use std::fs::File;
use std::path::PathBuf;
use bincode::serialize_into;
use std::io::BufWriter;

#[derive(Debug, StructOpt)]
#[structopt(name = "example")]
struct Args {
    /// The epression to generate geometry for
    #[structopt(short = "e", long = "expression")]
    expression: String,

    /// The epsilon value that serves as the basecase for our octtree recursion
    #[structopt(short = "s", long = "epsilon")]
    epsilon: f32,

    /// The file to write out output to
    #[structopt(name = "FILE", short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// If passed, we will create a new asap instance for the output file
    #[structopt(short = "p", long = "plot")]
    plot: bool,

    /// The bounding box side size
    #[structopt(short = "b", long = "bounding-box", default_value = "40.0")]
    box_size: f32,
}

fn main() {
        let args = Args::from_args();

        println!("Parsing...");
        let input: Vec<char> = args.expression.chars().collect();
        let f = implicit::parser::parse_expression(&input, 0).expect("Unable to parse expression");

        println!("Making mtree...");
        let size_interval = Interval::new(-args.box_size / 2.0, args.box_size / 2.0 );
        let bounding_box = BoundingBox {
            x: size_interval.clone(),
            y: size_interval.clone(),
            z: size_interval.clone(),
        };
    
        let mut bindings = HashMap::new();
        bindings.insert('x', bounding_box.x);
        bindings.insert('y', bounding_box.y);
        bindings.insert('z', bounding_box.z);

        // Bootstrap the mtree
        let intervals = f.evaluate_interval(&bindings);
        let mut n = MNode {
            intervals: intervals,
            bb: bounding_box,
            children: None,
        };
    
        n.find_roots(&f, args.epsilon);
   
        println!("Plotting mtree...");
        let mut plot = Plot::new();
        n.add_to_plot(false, &mut plot);

        let file = File::create(&args.output).unwrap();
        let mut w = BufWriter::new(file);

        serialize_into(&mut w, &plot).expect("Unable to serialize plot");

        if args.plot {
            println!("Calling plotter...");
            Command::new("/Users/russell/.cargo/bin/asap")
                .args(&[&args.output.as_os_str()])
                .output()
                .expect("failed to execute process");
        }
}
