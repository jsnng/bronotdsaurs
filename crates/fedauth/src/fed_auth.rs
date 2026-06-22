use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum FedAuthPluginError {
    Spawn(std::io::Error),
    Io(std::io::Error),
    Timeout,
    ExitStatus(Option<i32>),
    EmptyToken,
}

impl core::fmt::Display for FedAuthPluginError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spawn(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Timeout => write!(f, ""),
            Self::ExitStatus(Some(e)) => write!(f, "{e}"),
            Self::ExitStatus(None) => write!(f, ""),
            Self::EmptyToken => write!(f, ""),
        }
    }
}

impl std::error::Error for FedAuthPluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(e) | Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

pub struct FedAuthPlugin {
    pub binary: PathBuf,
    pub sts_url: String,
    pub spn: String,
    pub nonce: Option<[u8; 32]>,
    pub timeout: Duration,
}

impl FedAuthPlugin {
    pub fn acquire(&self) -> Result<Vec<u8>, FedAuthPluginError> {
        let mut demon = Command::new(&self.binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(FedAuthPluginError::Spawn)?;

        {
            let mut stdin = demon.stdin.take().expect("");
            let w = |s: &mut dyn Write, b: &[u8]| -> std::io::Result<()> {
                s.write_all(&(b.len() as u32).to_le_bytes())?;
                s.write_all(b)
            };
            let ok = w(&mut stdin, self.sts_url.as_bytes())
                .and_then(|_| w(&mut stdin, self.spn.as_bytes()))
                .and_then(|_| match &self.nonce {
                    Some(n) => {
                        stdin.write_all(&[1])?;
                        stdin.write_all(n)
                    }
                    None => stdin.write_all(&[0]),
                });
            if let Err(e) = ok {
                let _body = demon.kill();
                let _crime_scene = demon.wait();
                return Err(FedAuthPluginError::Io(e));
            }
        }

        let mut stdout = demon.stdout.take().expect("");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut token = Vec::new();
            let _tx = tx.send(stdout.read_to_end(&mut token).map(|_| token));
        });

        match rx.recv_timeout(self.timeout) {
            Ok(Ok(token)) => {
                let exit_status = demon.wait().map_err(FedAuthPluginError::Io)?;
                if !exit_status.success() {
                    return Err(FedAuthPluginError::ExitStatus(exit_status.code())); // code() is Option<i32> — exact match
                }
                if token.is_empty() {
                    return Err(FedAuthPluginError::EmptyToken);
                }
                Ok(token)
            }
            Ok(Err(e)) => {
                let _body = demon.kill();
                let _crime_scene = demon.wait();
                Err(FedAuthPluginError::Io(e))
            }
            _ => {
                let _body = demon.kill();
                let _crime_scene = demon.wait();
                Err(FedAuthPluginError::Timeout)
            }
        }
    }
}
