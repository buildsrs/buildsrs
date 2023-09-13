use tokio_postgres::{connect, NoTls};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(long, short, env)]
    pub database: String,
}

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let (mut client, connection) = connect(&options.database, NoTls).await.unwrap();
    tokio::spawn(connection);
    buildsrs_database::migrations::runner()
        .run_async(&mut client)
        .await
        .unwrap();
}
