use std::{result, error};
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::io::{Error as IoError};
use core::error::{Error as MailError};

pub type Result<T, E> = result::Result<T, Error<E>>;


#[derive(Debug)]
pub enum Error<RE: error::Error> {
    UnknownTemplateId(String),
    CIdGenFailed(MailError),
    RenderError(RE),
}


impl<R> error::Error for Error<R>
    where R: error::Error
{

    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            UnknownTemplateId(_) => "unknown template id",
            CIdGenFailed(_) => "generating a cid failed",
            RenderError(ref er) => er.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match *self {
            RenderError(ref er) => er.cause(),
            CIdGenFailed(ref er) => er.cause(),
            _ => None
        }
    }
}

impl<R> Display for Error<R>
    where R: error::Error
{
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            UnknownTemplateId(ref id) => {
                write!(fter, "unknwon template id: {:?}", id)
            },
            CIdGenFailed(ref err) => {
                write!(fter, "generating cid failed:")?;
                err.fmt(fter)
            }
            RenderError(ref re) => <R as fmt::Display>::fmt(re, fter)
        }
    }
}


#[derive(Debug)]
pub enum SpecError {
    /// error if the path is not a valid string
    NonStringPath(PathBuf),
    MissingTypeInfo(String),
    //FEAT potentially change from Box<Error> to a concrete type
    BodyMediaTypeCreationFailure(Box<error::Error+'static>),
    ResourceMediaTypeCreationFailure(Box<error::Error+'static>),
    IoError(IoError),
    DuplicateEmbeddingName(String),
    NoSubTemplatesFound(PathBuf),
    TemplateFileMissing(PathBuf),
    NotAFile(PathBuf)
}

impl error::Error for SpecError {

    fn cause(&self) -> Option<&error::Error> {
        use self::SpecError::*;
        match *self {
            BodyMediaTypeCreationFailure(ref err) => Some(&**err),
            ResourceMediaTypeCreationFailure(ref err) => Some(&**err),
            IoError(ref err) => Some(err),
            _ => None
        }
    }
    fn description(&self) -> &str {
        use self::SpecError::*;
        match *self {
            NonStringPath(_) => "path must also be valid string",
            MissingTypeInfo(_) => "no type info included in settings for given type",
            BodyMediaTypeCreationFailure(_) => "creating a media type for a mime body failed",
            ResourceMediaTypeCreationFailure(_) =>
                "creating a media type for a Embedding/Attachment failed",
            IoError(_) => "a I/O-Error occurred",
            DuplicateEmbeddingName(_) =>
                "multiple embedding with the same in-template name where found",
            NoSubTemplatesFound(_) =>
                "template folder needs to contain at last one subtemplate e.g. text or html",
            TemplateFileMissing(_) =>
                "sub-template folder does not contain a template file (e.g. `mail.html`)",
            NotAFile(_) =>
                "template_file, embedding or attachment was not a file"
        }
    }
}

impl Display for SpecError {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        use self::SpecError::*;
        match *self {
            NonStringPath(ref path) => {
                write!(fter, "path must also be valid string, got: {}", path.display())
            },
            MissingTypeInfo(ref type_) => {
                write!(fter, "no type info in settings for: {:?}", type_)
            },
            BodyMediaTypeCreationFailure(ref err) => {
                write!(fter, "media type creation for body failed: {}", err)
            },
            ResourceMediaTypeCreationFailure(ref err) => {
                write!(fter, "media type creation for Embedding/Attachment failed: {}", err)
            },
            IoError(ref err) => {
                write!(fter, "I/O-Error: {}", err)
            },
            DuplicateEmbeddingName(ref e) => {
                write!(fter, "multiple embeddings with the in-template name {:?} where found", e)
            },
            NoSubTemplatesFound(ref dir) => {
                write!(fter, "template dir has to contain at last one sub-template. dir: {}",
                       dir.display())
            },
            TemplateFileMissing(ref dir) => {
                write!(fter, "sub-template folder does not contain a template file: {}",
                       dir.display())
            },
            NotAFile(ref path) => {
                write!(fter, "the template/embedding/attachment {} is not a file", path.display())
            }

        }
    }
}

impl From<IoError> for SpecError {
    fn from(val: IoError) -> Self {
        SpecError::IoError(val)
    }
}