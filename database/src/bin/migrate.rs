use buildsrs_database::{migrations, Database};
use clap::Parser;
use ssh_key::{HashAlg, PublicKey};
use std::path::PathBuf;
use tokio::fs::read_to_string;
use tokio_postgres::{connect, NoTls};

#[derive(Parser, Debug)]
pub struct Options {
    #[clap(long, short, env, global = true, default_value = "")]
    pub database: String,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Migrate,
    Builder {
        #[clap(subcommand)]
        command: BuilderCommand,
    },
}

#[derive(Parser, Debug)]
pub enum BuilderCommand {
    Add {
        #[clap(env)]
        public_key_file: PathBuf,
    },
    Remove {
        #[clap(env)]
        public_key_file: PathBuf,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse();
    let (mut client, connection) = connect(&options.database, NoTls).await.unwrap();
    tokio::spawn(connection);

    match options.command {
        Command::Migrate => {
            migrations::runner().run_async(&mut client).await?;
            return Ok(());
        }
        _ => {}
    }

    let database = Database::new(client).await?;

    match options.command {
        Command::Migrate => unreachable!(),
        Command::Builder { command } => match command {
            BuilderCommand::Add { public_key_file } => {
                let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
                let encoded = key.to_openssh()?;
                let fingerprint_sha256 = key.fingerprint(HashAlg::Sha256).to_string();
                let fingerprint_sha512 = key.fingerprint(HashAlg::Sha512).to_string();
                database
                    .builder_add(&encoded, &fingerprint_sha256, &fingerprint_sha512)
                    .await?;
            }
            BuilderCommand::Remove { public_key_file } => {
                let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
            }
        },
    }

    Ok(())
}
