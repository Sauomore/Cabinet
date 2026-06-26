use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "cabinet")]
#[command(about = "Cabinet — HSH 离散语义记忆检索系统 CLI")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 记忆库路径
    #[arg(short, long, global = true, default_value = "./agent_memory.db")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// 插入文档到记忆库
    Insert {
        /// 要插入的文本（或从文件读取）
        text: String,
        /// 从文件读取文本
        #[arg(short, long)]
        file: bool,
    },
    /// 查询记忆库
    Query {
        /// 查询文本
        text: String,
        /// 返回 Top-K 结果
        #[arg(short, long, default_value = "5")]
        top_k: usize,
        /// 最小匹配级别 (1-4)
        #[arg(short, long, default_value = "1")]
        min_level: u8,
    },
    /// 批量插入（从文本文件，每行一条）
    Batch {
        /// 输入文件路径
        input: PathBuf,
    },
    /// 查看统计信息
    Stats,
    /// 创建快照备份
    Snapshot {
        /// 输出路径
        output: PathBuf,
    },
    /// 编码演示（只编码不入库）
    Encode {
        /// 要编码的文本
        text: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    use cabinet_core::{Config, Memory, QueryOpts, Precision};

    match cli.command {
        Commands::Insert { text, file } => {
            let content = if file {
                std::fs::read_to_string(&text)?
            } else {
                text
            };

            let mut mem = Memory::open(Config::new(&cli.path))?;
            let start = Instant::now();
            let doc_id = mem.insert(&content)?;
            let elapsed = start.elapsed();

            println!("{} {}
            "inserted".green().bold(),
            "✓".green(),
            );
            println!("  doc_id: {}", doc_id.to_string().cyan());
            println!("  chars:  {}", content.len().to_string().cyan());
            println!("  time:   {:?}", elapsed);
        }

        Commands::Query { text, top_k, min_level } => {
            let mut mem = Memory::open(Config::new(&cli.path))?;
            let start = Instant::now();
            let results = mem.query(&text, QueryOpts::new().top_k(top_k).min_match_level(min_level))?;
            let elapsed = start.elapsed();

            println!("{} {} results for \"{}\" in {:?}",
                "🔍".bold(),
                results.len().to_string().yellow().bold(),
                text.cyan(),
                elapsed
            );

            if results.is_empty() {
                println!("  {}", "No results found.".dimmed());
            } else {
                for (i, r) in results.iter().enumerate() {
                    let level_str = match r.match_level {
                        4 => "EXACT".red().bold(),
                        3 => "CLUSTER".yellow().bold(),
                        2 => "CATEGORY".green(),
                        1 => "RELATED".blue(),
                        _ => "UNKNOWN".normal(),
                    };
                    println!("  {}. [{}] score={:.3} doc_id={}",
                        (i + 1).to_string().bold(),
                        level_str,
                        r.score,
                        r.doc_id.to_string().cyan()
                    );
                    if let Some(ref txt) = r.text {
                        let preview = if txt.len() > 80 { &txt[..80] } else { txt };
                        println!("     \"{}\"", preview.dimmed());
                    }
                }
            }
        }

        Commands::Batch { input } => {
            let lines: Vec<String> = std::fs::read_to_string(&input)?
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let mut mem = Memory::open(Config::new(&cli.path))?;
            let start = Instant::now();

            use indicatif::{ProgressBar, ProgressStyle};
            let pb = ProgressBar::new(lines.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
                    .progress_chars("#>-"),
            );

            for line in &lines {
                mem.insert(line)?;
                pb.inc(1);
            }
            pb.finish_with_message("done");

            let elapsed = start.elapsed();
            println!("{} Inserted {} documents in {:?}",
                "✓".green(),
                lines.len().to_string().cyan().bold(),
                elapsed
            );
            println!("  throughput: {:.1} docs/s", lines.len() as f64 / elapsed.as_secs_f64());
        }

        Commands::Stats => {
            let mem = Memory::open(Config::new(&cli.path))?;
            // 通过 scan 统计
            println!("{} {}",
                "📊".bold(),
                "Memory Statistics".bold().underline()
            );
            println!("  path:     {}", cli.path.display().to_string().cyan());
            println!("  backend:  SQLite");
            // 实际统计需要通过内部 API，这里简化展示
            println!("  (use `cabinet query` to test retrieval)");
        }

        Commands::Snapshot { output } => {
            let mem = Memory::open(Config::new(&cli.path))?;
            mem.snapshot(&output)?;
            println!("{} Snapshot saved to {}",
                "💾".green(),
                output.display().to_string().cyan()
            );
        }

        Commands::Encode { text } => {
            use cabinet_hsh::Encoder;
            let encoder = Encoder::new();
            let start = Instant::now();
            let codes = encoder.encode(&text)?;
            let elapsed = start.elapsed();

            println!("{} \"{}\" → {} HSH codes in {:?}",
                "🔢".bold(),
                text.cyan(),
                codes.len().to_string().yellow().bold(),
                elapsed
            );
            for (i, code) in codes.iter().enumerate() {
                println!("  [{:2}] feat=0x{:01X} sim=0x{:02X} abs=0x{:02X} | raw=0x{:05X}",
                    i,
                    code.feat(),
                    code.sim(),
                    code.abs(),
                    code.raw()
                );
            }
        }
    }

    Ok(())
}
