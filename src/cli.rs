use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "termpop")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long)]
    pub initial: Option<String>,

    #[arg(long)]
    pub title: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Daemon {
        #[arg(long)]
        install: bool,
    },
    Status,
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_args_is_open_mode() {
        let cli = Cli::parse_from(["termpop"]);
        assert!(cli.command.is_none());
        assert!(cli.initial.is_none());
        assert!(cli.title.is_none());
    }

    #[test]
    fn parse_initial_text() {
        let cli = Cli::parse_from(["termpop", "--initial", "hello"]);
        assert_eq!(cli.initial.as_deref(), Some("hello"));
    }

    #[test]
    fn parse_title() {
        let cli = Cli::parse_from(["termpop", "--title", "Commit Message"]);
        assert_eq!(cli.title.as_deref(), Some("Commit Message"));
    }

    #[test]
    fn parse_daemon_command() {
        let cli = Cli::parse_from(["termpop", "daemon"]);
        assert!(matches!(
            cli.command,
            Some(Command::Daemon { install: false })
        ));
    }

    #[test]
    fn parse_daemon_install() {
        let cli = Cli::parse_from(["termpop", "daemon", "--install"]);
        assert!(matches!(
            cli.command,
            Some(Command::Daemon { install: true })
        ));
    }

    #[test]
    fn parse_status_command() {
        let cli = Cli::parse_from(["termpop", "status"]);
        assert!(matches!(cli.command, Some(Command::Status)));
    }

    #[test]
    fn parse_stop_command() {
        let cli = Cli::parse_from(["termpop", "stop"]);
        assert!(matches!(cli.command, Some(Command::Stop)));
    }
}
