use clap::Parser;
use essam::gzip::{compress as gzip_compress, decompress as gzip_decompress};

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
    gzip_compress(input_path, output_path).map_err(anyhow::Error::from)
}

fn decompress(input_path: String, output_path: String) -> anyhow::Result<()> {
    gzip_decompress(input_path, output_path).map_err(anyhow::Error::from)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.op {
        Operation::Compress(args) => compress(args.input_path, args.output_path),
        Operation::Decompress(args) => decompress(args.input_path, args.output_path),
    }
}
