//! Command handlers.
use rust_i18n::t;

use anyhow::Result;
use crate::cli::{Cli, Commands};

pub async fn run(cli: Cli) -> Result<()> {
    crate::i18n::init();

    let command = cli.command.unwrap_or(Commands::Serve {
        addr: "127.0.0.1:3000".to_string(),
    });

    match command {
        Commands::Serve { addr } => serve(&addr).await,
        Commands::Service { action } => service(action).await,
        Commands::Agents => agents().await,
        Commands::Api { method, path, data } => api(method, &path, data).await,
        Commands::Migrate => migrate().await,
        Commands::Debug => debug().await,
    }
}

async fn serve(addr: &str) -> Result<()> {
    tracing::info!("{}", t!("cli.serve").to_string());
    let server = opencode_server::Server::new();
    server.serve(addr).await?;
    Ok(())
}

async fn service(action: crate::cli::ServiceAction) -> Result<()> {
    use crate::cli::ServiceAction;
    match action {
        ServiceAction::Start => println!("{}", t!("cli.service.start").to_string()),
        ServiceAction::Restart => println!("{}", t!("cli.service.restart").to_string()),
        ServiceAction::Status => println!("{}", t!("cli.service.status").to_string()),
        ServiceAction::Stop => println!("{}", t!("cli.service.stop").to_string()),
    }
    Ok(())
}

async fn agents() -> Result<()> {
    let server = opencode_server::Server::new();
    let agents = server.state().agents.list().await;
    for agent in agents {
        println!("{} - {}", agent.id, agent.description.unwrap_or_default());
    }
    Ok(())
}

async fn api(method: Option<String>, path: &str, data: Option<String>) -> Result<()> {
    let method = method.unwrap_or_else(|| "GET".to_string());
    println!("{} {}", method, path);
    if let Some(d) = data {
        println!("Body: {}", d);
    }
    Ok(())
}

async fn migrate() -> Result<()> {
    println!("{}", t!("cli.migrate").to_string());
    Ok(())
}

async fn debug() -> Result<()> {
    println!("{}", t!("cli.debug.description").to_string());
    Ok(())
}
