//! Manages benchmarks commands to run the given benchmark or list all of the available ones.

mod core;

use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
/// Represents benchmarks commands to run the given benchmark or list all of the available ones.
pub enum BenchTarget {
    /// Run benchmarks in Rust Core
    Core {
        /// Name of the benchmark as stated in configurations file.
        #[arg(index = 1, required = true)]
        name: String,

        /// Path or configurations of the input source to be used in the benchmarks
        #[arg(short, long = "input")]
        input_source: Option<String>,

        /// Additional configurations for the benchmarks
        #[arg(short, long)]
        config: Option<String>,

        /// Determines how many times to run the benchmark.
        #[arg(short, long, default_value = "1")]
        run_count: u8,

        /// Sets sample size on criterion benchmarks
        #[arg(short, long)]
        sample_size: Option<usize>,
    },
    /// Lists all the available benchmarks in configuration files.
    List,
    /// Runs all benchmarks, optionally filtering by `ci_ignore`.
    RunAll {
        /// Include benchmarks with `ci_ignore = true`
        #[arg(long, default_value = "false")]
        ci_ignore: bool,
    },
}

impl BenchTarget {
    /// Runs the operations defined by each [`BenchTarget`]
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            BenchTarget::Core {
                name,
                input_source,
                config,
                run_count,
                sample_size,
            } => {
                core::run_benchmark(name, input_source, config, run_count, sample_size)?;
            }
            BenchTarget::List => {
                println!("Listing all benchmarks from configurations...");
                println!();

                let core = core::ConfigsInfos::load()?;
                core.print_list();
                println!();
            }
            BenchTarget::RunAll { ci_ignore } => {
                let core = core::ConfigsInfos::load()?;

                for bench in core.benches.iter() {
                    if !ci_ignore && bench.ci_ignore.unwrap_or(false) {
                        continue;
                    }

                    println!("Running benchmark: {}", bench.name);
                    let input = bench.default_input.clone();
                    let sample_size = bench.default_sample_size;

                    core::run_benchmark(
                        bench.name.clone(),
                        input,
                        None, // No additional config in `run-all`
                        1,    // Default to one run per benchmark
                        sample_size,
                    )?;
                }
            }
        }

        Ok(())
    }
}
