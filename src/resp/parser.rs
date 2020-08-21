use super::data::RespObj;

pub fn parse(buf: &[u8]) -> Result<(RespObj, usize), String> {
    match buf.first() {
        Some(b'*') => parse_array(&buf),
        Some(b'+') => parse_string(&buf),
        Some(b':') => parse_integer(&buf),
        Some(b'$') => parse_bulk_string(&buf),
        t => Err(format!("Parsing not implemented for: {:?}", t)),
    }
}

fn parse_string(buf: &[u8]) -> Result<(RespObj, usize), String> {
    let pos = match buf.iter().position(|&byte| byte == b'\r') {
        Some(pos) => pos,
        None => return Err(String::from("Failed to find terminating char")),
    };
    
    let result = simple_parse::<String>(&buf[1..pos])?;
    Ok((RespObj::String(result), pos + 2))
}


fn parse_integer(buf: &[u8]) -> Result<(RespObj, usize), String> {
    let pos = match buf.iter().position(|&byte| byte == b'\r') {
        Some(pos) => pos,
        None => return Err(String::from("Failed to find terminating char")),
    };
    
    let result = simple_parse::<i64>(&buf[1..pos])?;
    Ok((RespObj::Integer(result), pos + 2))
}

fn parse_bulk_string(buf: &[u8]) -> Result<(RespObj, usize), String> {
    let pos = match buf.iter().position(|&byte| byte == b'\r') {
        Some(pos) => pos,
        None => return Err(String::from("Failed to find terminating char")),
    };
    
    let size = simple_parse::<usize>(&buf[1..pos])?;
    let c_pos = pos + 2; // skip '\n', move to start of the string
    let content = simple_parse::<String>(&buf[c_pos..(c_pos + size)])?;
    
    Ok((RespObj::String(content), c_pos + size + 2))
}

fn parse_array(buf: &[u8]) -> Result<(RespObj, usize), String> {
    let pos = match buf.iter().position(|&byte| byte == b'\r') {
        Some(pos) => pos,
        None => return Err(String::from("Failed to find terminating char")),
    };
    
    let element_count = simple_parse::<usize>(&buf[1..pos])?;
    let mut res = vec![];
    let mut i = pos + 2;
    while res.len() < element_count {
        let (content, read_size) = parse(&buf[i..])?;
        res.push(content);
        i += read_size;
    }
    
    Ok((RespObj::Array(res), i))
}

fn simple_parse<T: std::str::FromStr>(buf: &[u8]) -> Result<T, String> {
    if let Ok(size_str) = String::from_utf8(buf.to_vec()) {
        match size_str.parse::<T>() {
            Ok(size) => return Ok(size),
            _ => ()
        }
    }
    
    Err(format!("Failed to parse {} from buffer: {:?}", std::any::type_name::<T>(), buf))
}
