use tokio_postgres::{connect, NoTls};

#[tokio::main]
async fn main() {
    let host = std::env::args()
        .skip(1)
        .map(|arg| format!("{arg} "))
        .collect::<String>();
    let (mut client, connection) = connect(&host, NoTls).await.unwrap();
    tokio::spawn(connection);
    buildsrs_database::migrations::runner()
        .run_async(&mut client)
        .await
        .unwrap();
}
