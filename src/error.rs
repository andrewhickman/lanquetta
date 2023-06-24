use std::io;

use druid::ArcStr;
use tonic::{Code, Status};

pub fn fmt_err(err: &anyhow::Error) -> ArcStr {
    use std::fmt::Write;

    let mut s = String::new();
    for cause in err.chain() {
        if !s.is_empty() {
            s.push_str(": ");
        }
        let len = s.len();
        write!(s, "{}", cause).unwrap();
        if s[..len].contains(&s[len..]) {
            s.truncate(len.saturating_sub(2));
            break;
        }
    }
    s.into()
}

pub fn fmt_connect_err(err: &anyhow::Error) -> ArcStr {
    if let Some(err) = err.root_cause().downcast_ref::<io::Error>() {
        format!("failed to connect: {}", err).into()
    } else {
        fmt_err(err)
    }
}

pub fn fmt_grpc_err(err: &anyhow::Error) -> ArcStr {
    if let Some(status) = err.downcast_ref::<Status>() {
        if status.message().is_empty() {
            fmt_code(status.code()).into()
        } else {
            format!("{}: {}", fmt_code(status.code()), status.message()).into()
        }
    } else {
        fmt_connect_err(err)
    }
}

fn fmt_code(code: Code) -> &'static str {
    match code {
        Code::Ok => "OK",
        Code::Cancelled => "CANCELLED",
        Code::Unknown => "UNKNOWN",
        Code::InvalidArgument => "INVALID_ARGUMENT",
        Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
        Code::NotFound => "NOT_FOUND",
        Code::AlreadyExists => "ALREADY_EXISTS",
        Code::PermissionDenied => "PERMISSION_DENIED",
        Code::ResourceExhausted => "RESOURCE_EXHAUSTED",
        Code::FailedPrecondition => "FAILED_PRECONDITION",
        Code::Aborted => "ABORTED",
        Code::OutOfRange => "OUT_OF_RANGE",
        Code::Unimplemented => "UNIMPLEMENTED",
        Code::Internal => "INTERNAL",
        Code::Unavailable => "UNAVAILABLE",
        Code::DataLoss => "DATA_LOSS",
        Code::Unauthenticated => "UNAUTHENTICATED",
    }
}
