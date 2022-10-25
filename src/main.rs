use netstring::ReadNetstring;
use netstring::Shutdown;
use serde::Deserialize;
use serde_json::Map;
use serde_json::Value;
use serde_json::from_str;
use serde_json::to_string;
use std::io::Error;
use std::io::ErrorKind::ConnectionAborted;
use std::io::ErrorKind::Unsupported;
use std::io::Read;
use std::io::Result;
use std::io::StdinLock;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
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
    let mut stdout = stdout().lock();

    loop {
        match stdin.read_netstring() {
            Ok(str) => {
                let o: Object = from_str(str.as_str())?;

                write_icinga_object(&mut stdout, &o)?;
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

fn write_icinga_object(w: &mut impl Write, o: &Object) -> Result<()> {
    w.write("object ".as_ref())?;
    w.write(o.r#type.as_bytes())?;
    w.write(" ".as_ref())?;
    write_icinga_str(w, &o.name)?;
    w.write(" {\n".as_ref())?;

    for (attr, val) in o.properties.iter() {
        match attr.as_str() {
            "__name" => {}
            "name" => {}
            "package" => {}
            "source_location" => {}
            "templates" => {}
            "type" => {}
            _ => {
                w.write("  ".as_ref())?;
                w.write(attr.as_bytes())?;
                w.write(" = ".as_ref())?;
                write_icinga_val(w, val)?;
                w.write("\n".as_ref())?;
            }
        }
    }

    w.write("}\n\n".as_ref())?;

    Ok(())
}

fn write_icinga_val(w: &mut impl Write, v: &Value) -> Result<()> {
    match v {
        Value::String(s) => { write_icinga_str(w, s)?; }

        Value::Array(arr) => {
            w.write("[ ".as_ref())?;

            for vv in arr.iter() {
                write_icinga_val(w, vv)?;
                w.write(", ".as_ref())?;
            }

            w.write("]".as_ref())?;
        }

        Value::Object(obj) => {
            w.write("{ ".as_ref())?;

            for (k, v) in obj.iter() {
                write_icinga_str(w, k)?;
                w.write(" = ".as_ref())?;
                write_icinga_val(w, v)?;
                w.write("; ".as_ref())?;
            }

            w.write("}".as_ref())?;
        }

        _ => { w.write(to_string(v)?.as_bytes())?; }
    }

    Ok(())
}

fn write_icinga_str(w: &mut impl Write, s: &String) -> Result<()> {
    w.write("{{{".as_ref())?;
    w.write(s.as_bytes())?;
    w.write("}}}".as_ref())?;

    Ok(())
}
