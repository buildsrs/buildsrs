use buildsrs_database::{migrations, Database};
use clap::Parser;
use ssh_key::{HashAlg, PublicKey};
use std::path::PathBuf;
use tokio::fs::read_to_string;
use tokio_postgres::{connect, NoTls};
use uuid::Uuid;

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
    Target {
        #[clap(subcommand)]
        command: TargetCommand,
    },
}

#[derive(Parser, Debug)]
pub enum BuilderCommand {
    Add {
        #[clap(env)]
        public_key_file: PathBuf,

        #[clap(long, env, default_value = "")]
        comment: String,
    },
    Edit {
        #[clap(env)]
        public_key_file: PathBuf,

        /// Set builder enabled.
        #[clap(long, env)]
        enabled: Option<bool>,

        /// Set comment.
        #[clap(long, env)]
        comment: Option<String>,

        /// Adds allowed target.
        #[clap(long, env)]
        target_add: Vec<String>,

        /// Removes allowed target.
        #[clap(long, env)]
        target_remove: Vec<String>,
    },
    List,
}

#[derive(Parser, Debug)]
pub enum TargetCommand {
    Add {
        #[clap(env)]
        target: String,
    },
    Edit {
        #[clap(env)]
        target: String,

        #[clap(long, env)]
        enabled: Option<bool>,
    },
    List,
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

    let mut database = Database::new(client).await?;

    match options.command {
        Command::Migrate => unreachable!(),
        Command::Builder { command } => match command {
            BuilderCommand::Add {
                public_key_file,
                comment,
            } => {
                let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
                let transaction = database.transaction().await?;
                transaction
                    .builder_add(Uuid::new_v4(), &key, &comment)
                    .await?;
                transaction.commit().await?;
            }
            BuilderCommand::Edit {
                public_key_file,
                enabled,
                comment,
                target_add,
                target_remove,
            } => {
                let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
                let builder = database
                    .builder_lookup(&key.fingerprint(HashAlg::Sha512).to_string())
                    .await?;
                let transaction = database.transaction().await?;
                if let Some(enabled) = enabled {
                    transaction.builder_set_enabled(builder, enabled).await?;
                }

                if let Some(comment) = comment {
                    transaction.builder_set_comment(builder, &comment).await?;
                }

                for target in &target_add {
                    transaction.builder_target_add(builder, &target).await?;
                }

                for target in &target_remove {
                    transaction.builder_target_remove(builder, &target).await?;
                }

                transaction.commit().await?;
            }
            BuilderCommand::List => {
                let transaction = database.transaction().await?;
                let builders = transaction.builder_list().await?;
                for builder in &builders {
                    let info = transaction.builder_get(*builder).await?;
                    println!("{info:#?}");
                }
            }
        },
        Command::Target { command } => match command {
            TargetCommand::Add { target } => {
                database.target_add(&target).await?;
            }
            TargetCommand::Edit { target, enabled } => {}
            TargetCommand::List => {
                let transaction = database.transaction().await?;
                for target in &transaction.target_list().await? {
                    let info = transaction.target_info(&target).await?;
                    println!("{target:#?}");
                }
            }
        },
    }

    Ok(())
}
