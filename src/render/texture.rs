use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    io::{Read, Write},
};

use byteorder::LE;
use image::{EncodableLayout, GenericImageView};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::{
    content::resouces::Storable,
    error::{FormatError, ResourceError},
    util::UninitVec,
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
    num_enum::TryFromPrimitive,
)]
#[repr(u8)]
pub enum TextureRepr {
    RGBA8,
}

impl TextureRepr {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            TextureRepr::RGBA8 => 4,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            TextureRepr::RGBA8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        }
    }
}

impl AsRef<str> for TextureRepr {
    fn as_ref(&self) -> &str {
        match self {
            TextureRepr::RGBA8 => "RGBA8",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureResource {
    pub size: (u32, u32),
    pub hash: u64,
    pub repr: TextureRepr,
    /// Texture data in rgba8.
    pub data: Vec<u8>,
}

impl Debug for TextureResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("size", &self.size)
            .field("data", &format!("[ {} KiB... ]", self.data.len() / 1024))
            .field("repr", &self.repr.as_ref())
            .field("hash", &format!("{:X}", self.hash))
            .finish()
    }
}

impl Hash for TextureResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.hash.hash(state);
    }
}
impl PartialEq for TextureResource {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.hash == other.hash
    }
}
impl Eq for TextureResource {}

impl Storable for TextureResource {
    fn save<W: Write>(&self, out: &mut W) -> Result<(), ResourceError> {
        use byteorder::WriteBytesExt;

        out.write_u32::<LE>(self.size.0)?;
        out.write_u32::<LE>(self.size.1)?;
        out.write_u64::<LE>(self.hash)?;
        out.write_u8(self.repr as u8)?;
        out.write(&self.data.as_bytes())?;
        Ok(())
    }

    fn load<R: Read>(source: &mut R) -> Result<Self, ResourceError> {
        use byteorder::ReadBytesExt;

        let w = source.read_u32::<LE>()?;
        let h = source.read_u32::<LE>()?;
        let hash = source.read_u64::<LE>()?;

        let format = source.read_u8()?;
        let format =
            TextureRepr::try_from(format).map_err(|_| FormatError::InvalidTextureFormat(format))?;

        let bpp = format.bytes_per_pixel();
        let mut data = unsafe { Vec::new_uninit(w as usize * h as usize * bpp) };
        source.read_exact(data.as_mut_slice())?;
        let data_hash = hash_data(data.as_bytes());

        if data_hash != hash {
            return Err(ResourceError::InvalidHash);
        }

        Ok(TextureResource {
            size: (w, h),
            hash,
            repr: format,
            data,
        })
    }
}

impl TextureResource {
    pub fn rgba8_from_memory(bytes: impl AsRef<[u8]>) -> TextureResource {
        let image = image::load_from_memory(MENU_ICONS).unwrap();
        let data = image.to_rgba8().as_bytes().to_vec();

        TextureResource {
            size: image.dimensions(),
            hash: hash_data(data.as_bytes()),
            repr: TextureRepr::RGBA8,
            data,
        }
    }

    pub fn descriptor(&self) -> wgpu::TextureDescriptor {
        let size = wgpu::Extent3d {
            width: self.size.0,
            height: self.size.1,
            depth_or_array_layers: 1,
        };

        wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.repr.format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        }
    }

    pub fn create_texture(&self, device: &wgpu::Device) -> wgpu::Texture {
        let texture = device.create_texture(&self.descriptor());
    }

    pub fn create_texture_and_upload(
        &self,
        device: &wgpu::Device,
        q: &wgpu::Queue,
    ) -> TextureReference {
        let texture = device.create_texture_with_data(q, &self.descriptor(), self.data.as_bytes());

        TextureReference {
            source: Some(self),
            texture: Some(texture),
        }
    }
}

pub struct TextureReference<'a> {
    source: Option<&'a TextureResource>,

    texture: Option<wgpu::Texture>,
}

impl<'a> TextureReference<'a> {
    pub fn create_texture(&mut self, device: &wgpu::Device) -> wgpu::Texture {
        self.texture = Some(device.create_texture(&self.texture.descriptor()));
    }

    pub fn upload(&self, encoder: &mut wgpu::CommandEncoder) {}
}

impl<T: AsRef<[u8]>> From<T> for TextureResource {
    fn from(bytes: T) -> Self {
        TextureResource::rgba8_from_memory(bytes)
    }
}

fn hash_data(data: &[u8]) -> u64 {
    let mut hasher = fasthash::sea::Hasher64::new();
    data.hash(&mut hasher);
    hasher.finish()
}

pub static MENU_ICONS: &[u8] = include_bytes!("../../assets/menu_icons.png");
