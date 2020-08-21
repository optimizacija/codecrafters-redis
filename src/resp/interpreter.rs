use super::time;
use super::data::Database;
use super::data::RespObj;
use super::data::RespCommand;

// TODO instead of string, throw a custom parsing error
pub fn interpret(resp_obj: &RespObj, db: &Database) -> Result<String, String> {
    match resp_obj {
        RespObj::Array(vec) => {
            interpret_args(&vec, db)
        },
        RespObj::String(_) => {
            interpret_args(&[resp_obj.to_owned()], db)
        },
        _ => Err(format!("Unsupported interpreter type: {:?}", resp_obj)),
    }
}

fn interpret_args(vec: &[RespObj], db: &Database) -> Result<String, String> {
    match vec.first() {
        Some(RespObj::String(contained_string)) => {
            let cmd = RespCommand::from(&contained_string.as_str().to_lowercase())?;
            
            match cmd {
                RespCommand::Ping => handle_ping(&vec[1..]),
                RespCommand::Echo => handle_echo(&vec[1..]),
                RespCommand::Get => handle_get(&vec[1..], db),
                RespCommand::Set => handle_set(&vec[1..], db),
            }
        },
        e => Err(format!("Unsupported command type: {:?}", e)),
    }
}

fn handle_set(vec: &[RespObj], db: &Database) -> Result<String, String> {
    if vec.len() != 2 && vec.len() != 4 {
        return Err(format!("Invalid number of arguments {}", vec.len()));
    }
    
    let key = match &vec[0] {
        RespObj::String(key) => key,
        ro => return Err(format!("Invalid key type for: {:?}", ro)),
    };

    let value = match &vec[1] {
        RespObj::String(value) => value,
        ro => return Err(format!("Invalid value type for: {:?}", ro)),
    };
    
    let expiry = if vec.len() == 4 {
        match &vec[3] {
            RespObj::Integer(expiry) => Some(*expiry as u128),
            RespObj::String(expiry) => expiry.parse::<u128>().ok(),
            _ => None
        }
    } else {
        None
    };
    
    match db.lock() {
        Ok(mut db) => {
            let expiry_timestamp = if let Some(expiry) = expiry {
                Some(time::get_millis_since_epoch() + expiry)
            } else {
                None
            };
            
            db.insert(key.clone(), (value.clone(), expiry_timestamp));
            Ok("+OK\r\n".into())
        }
        Err(e) => {
            Err(format!("Failed to acquire db lock: {:?}", e))
        },
    }
}

fn handle_get(vec: &[RespObj], db: &Database) -> Result<String, String> {
    if vec.len() != 1 {
        return Err(format!("Invalid number of arguments {}", vec.len()));
    }
    
    let key = match &vec[0] {
        RespObj::String(key) => key,
        ro => return Err(format!("Invalid key: {:?}", ro)),
    };
    
    match db.lock() {
        Ok(mut db) => {
            if let Some((value, expiry)) = db.get(key) {
                if let Some(expiry) = expiry {
                    let timestamp = time::get_millis_since_epoch();
                    if timestamp < *expiry {
                        Ok(format!("+{}\r\n", value))
                    } else {
                        db.remove(key);
                        Ok("$-1\r\n".into())
                    }
                } else {
                    Ok(format!("+{}\r\n", value))
                }
            } else {
                Ok("+(nil)\r\n".into())
            }
        }
        Err(e) => {
            Err(format!("Failed to acquire db lock: {:?}", e))
        },
    }
}

fn handle_echo(vec: &[RespObj]) -> Result<String, String> {
    if vec.is_empty() {
        Ok("".into())
    } else {
        if let Some(RespObj::String(arg)) = vec.first() {
            Ok(format!("+{}\r\n", arg))
        } else {
            Err(format!("Unsupported argument for PING command {:?}", vec.first()))
        }
    }
}

fn handle_ping(vec: &[RespObj]) -> Result<String, String> {
    if vec.is_empty() {
        Ok("+PONG\r\n".to_string())
    } else {
        if let Some(RespObj::String(arg)) = vec.first() {
            Ok(format!("+PONG {}\r\n", arg))
        } else {
            Err(format!("Unsupported argument for PING command {:?}", vec.first()))
        }
    }
}

