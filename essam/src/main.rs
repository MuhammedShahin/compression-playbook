use clap::Parser;
use essam::deflate::{
    compress as deflate_compress, decompress as deflate_decompress, DeflateOptions,
};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone, clap::Args)]
struct OperationArgs {
    input_path: String,
    output_path: String,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Operation {
    Compress(OperationArgs),
    Decompress(OperationArgs),
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(subcommand)]
    op: Operation,
}

fn compress(input_path: String, output_path: String) -> anyhow::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let buf_reader = BufReader::new(input_file);
    let buf_writer = BufWriter::new(output_file);

    deflate_compress(buf_reader, buf_writer, DeflateOptions::default()).map_err(anyhow::Error::from)
}

fn decompress(input_path: String, output_path: String) -> anyhow::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let buf_reader = BufReader::new(input_file);
    let buf_writer = BufWriter::new(output_file);

    deflate_decompress(buf_reader, buf_writer).map_err(anyhow::Error::from)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.op {
        Operation::Compress(args) => compress(args.input_path, args.output_path),
        Operation::Decompress(args) => decompress(args.input_path, args.output_path),
    }
}
