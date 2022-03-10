// Copyright (c) 2022 MASSA LABS <info@massa.net>

use displaydoc::Display;
use thiserror::Error;

#[non_exhaustive]
#[derive(Display, Error, Debug)]
pub enum MassaHashError {
    /// parsing error : {0}
    ParsingError(String),

    /// Wrong prefix for hash: expected {0}, got {1}
    WrongPrefix(String, String),
}
