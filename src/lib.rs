use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;


// コマンドの引数、オプションを格納する構造体
#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}


// コマンドに与えられた引数、オプションを解析し、Config構造体を返す
// 実例：catr -n hoge.txt fuga.txt
// -> Config(files: ["hoge.txt", "fuga.txt"], number_lines: true, number_nonblank_lines: false)
pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Ken youens-Clark <kyclark@gmail.com>")
        .about("Rust cat")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input files")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number_lines")
                .short("n")
                .long("number")
                .help("number all output lines")
                .takes_value(false)
                .conflicts_with("number_nonblank_lines") // number_linesとnumber_nonblank_linesの同時指定は不可とする
        )
        .arg(
            Arg::with_name("number_nonblank_lines")
                .short("b")
                .long("number-nonblank")
                .help("number nonempty output lines")
                .takes_value(false)
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number_lines"),
        number_nonblank_lines: matches.is_present("number_nonblank_lines")
    })
}


// 引数に"-"が与えられた場合、標準入力を読み込み
// それ以外の文字列（パス形式）が与えられた場合、そのファイルパスを読み込む
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}


// メインの処理（ファイルの各行/行番号の出力）を実行する
pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),

            Ok(file) => {
                // number_nonblank_linesオプションで表示する行番号を保持する
                let mut current_nonblank_line_num = 0;

                for (line_index, line) in file.lines().enumerate() {
                    let line = line?;

                    // 与えられたオプションによって処理を分岐する
                    // 行番号を表示 *空行も含め附番*
                    if config.number_lines {
                        println!("{:>6}\t{}", line_index + 1, line);
                        continue;
                    }

                    // number_nonblank_lines => *空行ではない行に附番*
                    if config.number_nonblank_lines {
                        if !line.is_empty() {
                            current_nonblank_line_num += 1;
                            println!("{:>6}\t{}", current_nonblank_line_num, line);
                        } else {
                            println!();
                        }
                        continue;
                    }

                    // オプションが無し => 行番号は表示せず、行の文字列をそのまま表示
                    println!("{}", line);
                }
            }
        }
    }
    Ok(())
}
