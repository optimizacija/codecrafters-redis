use std::{sync::{Arc, Mutex}, collections::HashMap};

use super::parser;

#[derive(Debug, Clone)]
pub enum RespObj {
    Array(Vec<RespObj>),
    String(String),
    Integer(i64),
}

impl RespObj {
    pub fn from(buf: &[u8]) -> Result<RespObj, String> {
        Ok(parser::parse(buf)?.0)
    }
}

#[derive(Debug)]
pub enum RespCommand {
    Ping,
    Echo,
    Get,
    Set,
}

impl RespCommand {
    pub fn from(source: &str) -> Result<Self, String> {
        match source {
            "ping" => Ok(Self::Ping),
            "echo" => Ok(Self::Echo),
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            _ => Err(format!("Unrecognized command {}", source)),
        }
    }
}

pub type Database =  Arc<Mutex<HashMap<String, (String, Option<u128>)>>>;
