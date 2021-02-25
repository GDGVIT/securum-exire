use tokio::process::Command;
use std::ffi::OsStr;
use tokio::io::AsyncReadExt;
use std::collections::{BTreeMap};


async fn read_file(p: &str) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>>{
    let mut f = tokio::fs::File::open(p).await?;
    let mut contents= String::new();
    f.read_to_string(&mut contents).await?;
    let lines_of_file = contents.split("\n").collect::<Vec<&str>>();
    let mut m = BTreeMap::new();
    for x in lines_of_file {
        let values = x.split("=").collect::<Vec<&str>>();
        if values.len() >= 2 {
            m.insert(values[0].to_string(), values[1].to_string());
        }
    }
    Ok(m)
}


#[tokio::main]
async fn main() {

    



    let home_dir_path = std::env::var("HOME");
    let m;
    if let Ok(x) = home_dir_path {
        m = x;
    } else {
        eprintln!("error while fetching HOME variable!");
        std::process::exit(1);
    }
    let cmd_line_inp = [
        OsStr::new(m.as_str()),
        OsStr::new("-iname"),
        OsStr::new(".env")
    ];
    let cmd = Command::new("find")
        .args(&cmd_line_inp)
        .output()
        .await;

    let mut h = BTreeMap::new();

    if let Ok(x) = cmd {
        if x.status.success() {
            let out = std::str::from_utf8(x.stdout.as_slice());
            if let Ok(soo) = out {
                let mut paths = soo.split("\n").collect::<Vec<&str>>();
                paths.resize(paths.len()-1, "");
                for p in paths {
                    let x = read_file(p).await;
                    if let Ok( s) = x {
                        s.into_iter().for_each(|(k, v)| {
                            h.insert(k, v);
                        });
                    }
                }
                println!("{:?}", serde_json::to_string(&h));
            } else {
                eprintln!("error while processing output from find!");
                std::process::exit(1);
            }
        }
    } else {
        println!("Error occurred: {:?}", cmd.err())
    }
}
