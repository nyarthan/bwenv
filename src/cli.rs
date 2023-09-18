use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        long_help = "access token for the service account",
        help = "access token for the service account",
        env = "BWS_ACCESS_TOKEN",
        required = false
    )]
    pub token: String,

    #[arg(
        short,
        long = "profile for loading project configuration",
        long_help = "profile for loading project configuration",
        env = "BWENV_PROFILE",
        required = false
    )]
    pub profile: Option<String>,

    #[arg(last = true)]
    pub slop: Vec<String>,
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Secret manager project name",
    //     required = false
    // )]
    // pub project: String,

    // #[arg(long, long_help = "Profile of the project", required = false)]
    // pub profile: String,
    //
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Cache directory for the secrets",
    //     required = false
    // )]
    // pub cache_dir: String,
    //
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Revalidate the cache after the giben number of seconds",
    //     default_value_t = 3600
    // )]
    // pub revalidate: u64,
}

pub struct Cli {
    pub args: Args,
}

impl Cli {
    pub fn new() -> Self {
        let args = Args::parse();

        Cli { args }
    }

    pub fn get_program(&self) -> Option<(String, Vec<String>)> {
        let slop = &self.args.slop;
        match &slop.get(0) {
            Some(program) => {
                let args = slop[1..].to_vec();

                Some((program.to_string(), args))
            }
            None => None,
        }
    }
}
