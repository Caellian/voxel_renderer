use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader, Read, Write},
    marker::StructuralEq,
    path::{Path, PathBuf},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use byteorder::LE;
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::{error::ResourceError, render::texture::TextureResource};

pub trait Storable: Sized {
    /// Saves resource into a byte sink
    fn save<W: Write>(&self, out: &mut W) -> Result<(), ResourceError>;
    /// Loads reasource from source
    fn load<R: Read>(source: &mut R) -> Result<Self, ResourceError>;
}

#[derive(Debug, EnumKind, Hash)]
#[enum_kind(
    ResourceKind,
    derive(num_enum::TryFromPrimitive, Serialize, Deserialize),
    repr(u8)
)]
pub enum AnyResource {
    Texture(TextureResource),
}

impl AnyResource {
    pub fn tag(&self) -> u8 {
        ResourceKind::from(self) as u8
    }
}

impl From<TextureResource> for AnyResource {
    fn from(t: TextureResource) -> Self {
        AnyResource::Texture(t)
    }
}

impl Storable for AnyResource {
    fn save<W: Write>(&self, out: &mut W) -> Result<(), ResourceError> {
        use byteorder::WriteBytesExt;

        out.write_u8(self.tag());
        match self {
            AnyResource::Texture(t) => t.save(out),
        }
    }

    fn load<R: Read>(source: &mut R) -> Result<Self, ResourceError> {
        use byteorder::ReadBytesExt;

        let tag = source.read_u8()?;
        let kind =
            ResourceKind::try_from(tag).map_err(|_| ResourceError::InvalidResourceKind(tag))?;

        Ok(match kind {
            ResourceKind::Texture => AnyResource::Texture(TextureResource::load(source)?),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourceBank<R: Storable> {
    source: PathBuf,
    paths: HashMap<PathBuf, usize, fasthash::city::Hash128>,
    resources: Vec<R>,
}

impl<R: Storable> ResourceBank<R> {
    pub fn new(path: impl AsRef<Path>) -> Self {
        ResourceBank::Path(path)
    }

    fn load(source: impl AsRef<Path>) -> Result<Self, ResourceError> {
        let f = File::open(source.as_ref())?;

        let mut buff = BufReader::new(f);
    }

    fn save(&self) -> Result<(), ResourceError> {
        let f = File::create(self.source.as_path())?;

        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            ResourceBank::Path(_) => false,
            ResourceBank::Loaded {
                source,
                resources,
                cleared,
            } => !cleared,
        }
    }

    pub fn unload(&mut self) {
        match self {
            ResourceBank::Loaded {
                source,
                resources,
                cleared,
            } => {
                *resources = *cleared = true;
            }
            cleared => cleared,
        }
    }

    #[cfg(feature = "authoring")]
    pub fn push(&mut self, path: impl AsRef<Path>, resource: R) {
        let i = self.resources.len();
        self.resources.push(resource);
        self.paths.insert(path.as_ref(), i);
    }

    #[cfg(feature = "authoring")]
    pub fn remove(&mut self, path: impl AsRef<Path>) -> Option<R> {
        let i = self.paths.remove(path.as_ref())?;
        self.paths = self
            .paths
            .into_iter()
            .map(|(path, n)| if n > i { (path, n - 1) } else { (path, n) })
            .collect();
        Some(self.resources.remove(i))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ResourceID(pub u64);

impl<P: AsRef<Path>> From<P> for ResourceID {
    fn from(path: P) -> Self {
        let mut hasher = fasthash::city::Hasher128::new();
        path.as_ref().hash(&mut hasher);
        ResourceID(hasher.finalize())
    }
}

pub type WritePoisonError<'a> =
    std::sync::PoisonError<RwLockWriteGuard<'a, ResourceBank<AnyResource>>>;

pub enum MaybeNotLoaded<'r, R: Storable> {
    NotLoaded {
        bank: &'r ResourceBank<R>,
        path: PathBuf,
    },
    Loaded {
        value: R,
    },
}
