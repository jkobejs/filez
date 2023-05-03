use filez::{live, Files};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let files = live(".".to_string());
    for path in files.list("src/**/*.rs")? {
        let content = files.read(&path).await?;
        println!("{path}:");
        println!("{}", content);
    }

    Ok(())
}
