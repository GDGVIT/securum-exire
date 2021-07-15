use clap::{App, Arg};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

async fn read_file(p: &str) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut f = tokio::fs::File::open(p).await?;
    let mut contents = String::new();
    f.read_to_string(&mut contents).await?;
    let lines_of_file = contents.split("\n").collect::<Vec<&str>>();
    let mut m = BTreeMap::new();
    for x in lines_of_file {
        let mut x = x.trim().to_string();
        let b = x.find("#");
        let c = x.find("\"");
        let d = x.rfind("\"");
        if let Some(b) = b {
            let halt_str_present = x.find("HALT_SCAN");
            if let Some(d) = halt_str_present {
                if d > b {
                    break;
                }
            }
            if c.is_some() && d.is_some() {
                let c = c.unwrap();
                let d = d.unwrap();
                if !(b > c && b < d) {
                    x = x.chars().take(b).collect::<String>();
                } else {
                    x = x.chars().take(d).collect::<String>();
                }
            } else {
                x = x.chars().take(b).collect::<String>();
            }
        }
        let values = x.split("=").collect::<Vec<&str>>();
        if values.len() >= 2 {
            let key = values[0];
            let value = values[1..].concat();
            let key = key.trim_matches(&['"'] as &[_]).to_string();
            let value = value.trim_matches(&['"'] as &[_]).to_string();
            m.insert(key, value);
        }
    }
    Ok(m)
}

async fn save_file(
    h: &BTreeMap<String, String>,
    n: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let z = serde_json::to_string(&h);
    let path = n + "/credentials.json";
    if let Ok(z) = z {
        let mut f = tokio::fs::File::create(path.clone()).await?;
        f.write_all(z.as_bytes()).await?;
        println!("Credentials written at path [{}]", path);
    } else {
        return Err(Box::new(simple_error::SimpleError::new(
            "error_occurred_while_marshaling",
        )));
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let home_dir_path = std::env::var("HOME");
    let mut m = String::new();
    let matches = App::new("Securum Exire")
        .version("1.0")
        .author("Mayank Kumar <mayank22oct@gmail.com>")
        .about("CLI to run crons searching for credentials on a system")
        .arg(
            Arg::new("out")
                .short('o')
                .long("out")
                .default_value(".")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .default_value("HOME")
                .required(false)
                .takes_value(true),
        )
        .get_matches();
    if let Some(p) = matches.value_of("path") {
        if p == "HOME" {
            if let Ok(x) = home_dir_path {
                m = x;
            } else {
                eprintln!("error while fetching HOME variable!");
                std::process::exit(1);
            }
        } else {
            m = p.to_string();
        }
    }
    let mut n = String::new();
    if let Some(p) = matches.value_of("out") {
        n = p.to_string();
    }

    let cmd_line_inp = [
        OsStr::new(m.as_str()),
        OsStr::new("-iname"),
        OsStr::new(".env"),
    ];
    let cmd = Command::new("find").args(&cmd_line_inp).output().await;

    let mut h = BTreeMap::new();

    if let Ok(x) = cmd {
        if x.status.success() {
            let out = std::str::from_utf8(x.stdout.as_slice());

            if let Ok(soo) = out {
                println!("{}", soo);
                let mut paths = soo.split("\n").collect::<Vec<&str>>();
                paths.resize(paths.len() - 1, "");
                for p in paths {
                    let x = read_file(p).await;
                    if let Ok(s) = x {
                        s.into_iter().for_each(|(k, v)| {
                            h.insert(k, v);
                        });
                    }
                }
                let res = save_file(&h, n).await;
                if res.is_err() {
                    println!("error occurred while saving the credentials.");
                    std::process::exit(1);
                }
            } else {
                eprintln!("error while processing output from find!");
                std::process::exit(1);
            }
        } else {
            std::process::exit(1);
        }
    } else {
        println!("Error occurred: {:?}", cmd.err())
    }
}
