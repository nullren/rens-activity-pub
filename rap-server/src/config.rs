use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Address to listen on
    #[arg(short, long, env, default_value = "0.0.0.0")]
    pub(crate) address: String,

    /// Port to listen on
    #[arg(short, long, env, default_value = "3000")]
    pub(crate) port: String,

    /// Domain to use for the server
    #[arg(short, long, env)]
    pub(crate) domain: String,
}
