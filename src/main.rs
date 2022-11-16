use clap::{Arg, Command};
use std::io::BufRead;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Command::new("RustBust")
        .author("numonce")
        .version("1.0.2")
        .about("A simple dirb clone in Async Rust!")
        .arg(
            Arg::new("wordlist")
                .takes_value(true)
                .required(true)
                .long("wordlist")
                .short('w')
                .help("Path to wordlist to be used"),
        )
        .arg(
            Arg::new("url")
                .takes_value(true)
                .required(true)
                .long("url")
                .short('u')
                .help("Url to be used"),
        )
        .get_matches();

    let cli = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()?;
    let urlbuff = app.get_one::<String>("url").unwrap();
    let wordfile = std::fs::File::open(app.get_one::<String>("wordlist").unwrap())?;
    let reader = std::io::BufReader::new(wordfile);
    let semaphore = Arc::new(Semaphore::new(120));
    let tasks: Vec<_> = reader
        .lines()
        .map(|word| {
            let newcli = cli.clone();
            let url = urlbuff.clone();
            let permit = semaphore.clone();
            tokio::spawn(async move {
                let _holder = permit.acquire().await.unwrap();
                request(url, word.unwrap(), newcli).await.unwrap();
            })
        })
        .collect();

    for task in tasks {
        task.await?;
    }
    Ok(())
}

async fn request(
    url: String,
    word: String,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/{}", url, word);
    let res = client.get(&url).send().await;
    if let Ok(res) = res {
        if res.status() != 404 {
            println!(" {} {}", url, res.status());
        }
    }

    Ok(())
}
