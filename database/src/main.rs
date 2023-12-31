#![allow(missing_docs)]
use buildsrs_database::{migrations, Database, Transaction};
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
    Triple {
        #[clap(subcommand)]
        command: TripleCommand,
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

        /// Adds allowed triple.
        #[clap(long, env)]
        triple_add: Vec<String>,

        /// Removes allowed triple.
        #[clap(long, env)]
        triple_remove: Vec<String>,
    },
    List,
}

#[derive(Parser, Debug)]
pub enum TripleCommand {
    Add {
        #[clap(env)]
        triple: String,
    },
    Edit {
        #[clap(env)]
        triple: String,

        #[clap(long, env)]
        enabled: Option<bool>,

        #[clap(long, env)]
        rename: Option<String>,
    },
    List,
}

impl Command {
    async fn apply(
        &self,
        database: &mut Database<Transaction<'_>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Migrate => unreachable!(),
            Command::Builder { command } => match command {
                BuilderCommand::Add {
                    public_key_file,
                    comment,
                } => {
                    let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
                    database.builder_add(Uuid::new_v4(), &key, comment).await?;
                }
                BuilderCommand::Edit {
                    public_key_file,
                    enabled,
                    comment,
                    triple_add,
                    triple_remove,
                } => {
                    let key = PublicKey::from_openssh(&read_to_string(&public_key_file).await?)?;
                    let builder = database
                        .builder_lookup(&key.fingerprint(HashAlg::Sha512).to_string())
                        .await?;
                    if let Some(enabled) = enabled {
                        database.builder_set_enabled(builder, *enabled).await?;
                    }

                    if let Some(comment) = comment {
                        database.builder_set_comment(builder, comment).await?;
                    }

                    for triple in triple_add {
                        database.builder_triple_add(builder, triple).await?;
                    }

                    for triple in triple_remove {
                        database.builder_triple_remove(builder, triple).await?;
                    }
                }
                BuilderCommand::List => {
                    let builders = database.builder_list().await?;
                    for builder in &builders {
                        let info = database.builder_get(*builder).await?;
                        println!("{info:#?}");
                    }
                }
            },
            Command::Triple { command } => match command {
                TripleCommand::Add { triple } => {
                    database.triple_add(triple).await?;
                }
                TripleCommand::Edit {
                    triple,
                    enabled,
                    rename,
                } => {
                    if let Some(enabled) = enabled {
                        database.triple_enabled(triple, *enabled).await?;
                    }

                    if let Some(rename) = rename {
                        database.triple_rename(triple, rename).await?;
                    }
                }
                TripleCommand::List => {
                    for triple in &database.triple_list().await? {
                        let _info = database.triple_info(triple).await?;
                        println!("{triple:#?}");
                    }
                }
            },
        }

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command-line options
    let options = Options::parse();

    // connect to database
    let (mut client, connection) = connect(&options.database, NoTls).await.unwrap();
    tokio::spawn(connection);

    // handle migration
    if let Command::Migrate = options.command {
        migrations::runner().run_async(&mut client).await?;
        return Ok(());
    }

    // create database handle, run command
    let mut database = Database::new(client).await?;
    let mut database = database.transaction().await?;
    options.command.apply(&mut database).await?;
    database.commit().await?;

    Ok(())
}
