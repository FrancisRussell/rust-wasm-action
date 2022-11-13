use crate::actions::{core, exec, io, tool_cache};
use crate::node;
use crate::Error;

#[derive(Debug)]
pub struct Rustup {
    path: String,
}

impl Rustup {
    pub async fn get_or_install() -> Result<Rustup, Error> {
        match Self::get().await {
            Ok(rustup) => Ok(rustup),
            Err(e) => {
                core::info(format!("Unable to find rustup: {:?}", e));
                core::info("Installing it now");
                Self::install().await
            }
        }
    }

    pub async fn get() -> Result<Rustup, Error> {
        io::which("rustup", true)
            .await
            .map(|path| Rustup { path })
            .map_err(Error::Js)
    }

    pub async fn install() -> Result<Rustup, Error> {
        let args = ["--default-toolchain", "none", "-y"];
        let platform = String::from(node::os::platform());
        match platform.as_str() {
            "darwin" | "linux" => {
                let rustup_script = tool_cache::download_tool(
                    "https://sh.rustup.rs",
                    &tool_cache::DownloadParams::default(),
                )
                .await
                .map_err(Error::Js)?;
                core::info(format!("Downloaded to: {:?}", rustup_script));
                node::fs::chmod(rustup_script.as_str(), 0x755)
                    .await
                    .map_err(Error::Js)?;
                exec::exec(rustup_script.as_str(), args)
                    .await
                    .map_err(Error::Js)?;
                let env = node::process::env();
                core::info(format!("Environment is {:?}", env));
                todo!("Add rust to path");
            }
            _ => panic!("Unsupported platform: {}", platform),
        }
        core::info(format!("Platform: {:?}", platform));
    }
}
