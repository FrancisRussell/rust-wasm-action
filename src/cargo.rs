use crate::actions::core::Annotation;
use crate::actions::exec::{Command, Stdio};
use crate::actions::io;
use crate::info;
use crate::node::path::Path;
use crate::Error;

#[derive(Debug)]
pub struct Cargo {
    path: Path,
}

impl Cargo {
    pub async fn from_environment() -> Result<Cargo, Error> {
        io::which("cargo", true)
            .await
            .map(|path| Cargo { path })
            .map_err(Error::Js)
    }

    fn process_json_record(line: &str) {
        use cargo_metadata::Message;

        let metadata: Message = match serde_json::from_str(line) {
            Ok(metadata) => metadata,
            Err(e) => {
                info!("Unable to cargo output line as JSON metadata record: {}", e);
                return;
            }
        };
        if let Message::CompilerMessage(compiler_message) = metadata {
            let diagnostic = compiler_message.message;
            let annotation = Annotation::from(diagnostic.message);
            annotation.warning();
        }
    }

    pub async fn run<'a, I>(&'a mut self, subcommand: &'a str, args: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let subcommand = subcommand.to_string();
        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        let mut final_args = Vec::new();
        final_args.push(subcommand.clone());
        let process_json = if subcommand == "clippy" {
            final_args.push("--message-format=json".into());
            true
        } else {
            false
        };
        final_args.extend(args);

        let mut command = Command::from(&self.path);
        command.args(final_args);
        if process_json {
            command.outline(|line| Self::process_json_record(line));
            command.stdout(Stdio::null());
        }
        command.exec().await.map_err(Error::Js)?;
        Ok(())
    }
}
