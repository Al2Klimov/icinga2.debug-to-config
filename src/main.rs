use netstring::ReadNetstring;
use netstring::Shutdown;
use serde::Deserialize;
use serde_json::Map;
use serde_json::Value;
use serde_json::from_str;
use std::io::Error;
use std::io::ErrorKind::ConnectionAborted;
use std::io::ErrorKind::Unsupported;
use std::io::Read;
use std::io::Result;
use std::io::StdinLock;
use std::io::stdin;
use std::net::Shutdown as ShutdownMode;

struct ShutdownableStdin(StdinLock<'static>);

impl Read for ShutdownableStdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl Shutdown for ShutdownableStdin {
    fn shutdown(&self, _how: ShutdownMode) -> Result<()> {
        Err(Error::from(Unsupported))
    }
}

#[derive(Deserialize)]
struct Object {
    r#type: String,
    name: String,
    properties: Map<String, Value>,
}

fn main() -> Result<()> {
    let mut stdin = ShutdownableStdin(stdin().lock());

    loop {
        match stdin.read_netstring() {
            Ok(str) => {
                let o: Object = from_str(str.as_str())?;
                println!("{} {}", o.r#type, o.name);
            }
            Err(err) => {
                return match err.kind() {
                    ConnectionAborted => Ok(()),
                    _ => Err(err),
                };
            }
        }
    }
}
