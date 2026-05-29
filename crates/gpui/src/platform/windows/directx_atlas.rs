use anyhow::{Result, anyhow, bail};
use collections::FxHashMap;
use etagere::BucketedAtlasAllocator;
use parking_lot::Mutex;
use windows::Win32::Graphics::{
    Direct3D11::{
        D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
        ID3D11Device, ID3D11DeviceContext, ID3D11ShaderResourceView, ID3D11Texture2D,
    },
    Dxgi::Common::*,
};

use crate::{
    AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, Bounds, DevicePixels, PlatformAtlas,
    Point, Size, VideoTextureId, platform::AtlasTextureList,
};

pub(crate) struct DirectXAtlas(Mutex<DirectXAtlasState>);

struct DirectXAtlasState {
    device: ID3D11Device,
    device_context: ID3D11DeviceContext,
    monochrome_textures: AtlasTextureList<DirectXAtlasTexture>,
    polychrome_textures: AtlasTextureList<DirectXAtlasTexture>,
    video_textures: Vec<DirectXVideoTextureSlot>,
    video_free_list: Vec<usize>,
    tiles_by_key: FxHashMap<AtlasKey, AtlasTile>,
}

struct DirectXAtlasTexture {
    id: AtlasTextureId,
    bytes_per_pixel: u32,
    allocator: BucketedAtlasAllocator,
    texture: ID3D11Texture2D,
    view: [Option<ID3D11ShaderResourceView>; 1],
    live_atlas_keys: u32,
}

struct DirectXVideoTexture {
    width: u32,
    height: u32,
    texture: ID3D11Texture2D,
    view: [Option<ID3D11ShaderResourceView>; 1],
}

struct DirectXVideoTextureSlot {
    generation: u32,
    texture: Option<DirectXVideoTexture>,
}

fn next_video_generation(generation: u32) -> u32 {
    let next = generation.wrapping_add(1);
    if next == 0 { 1 } else { next }
}

fn video_frame_byte_len(width: u32, height: u32) -> Result<usize> {
    const MAX_VIDEO_TEXTURE_SIZE: u32 = 16384;

    if width == 0 || height == 0 {
        bail!("video texture dimensions must be non-zero");
    }
    if width > MAX_VIDEO_TEXTURE_SIZE || height > MAX_VIDEO_TEXTURE_SIZE {
        bail!(
            "video texture dimensions must be at most {}x{} on DirectX",
            MAX_VIDEO_TEXTURE_SIZE,
            MAX_VIDEO_TEXTURE_SIZE
        );
    }

    let byte_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|len| len.checked_mul(4))
        .ok_or_else(|| anyhow!("video frame dimensions overflow byte length"))?;

    usize::try_from(byte_len).map_err(|_| anyhow!("video frame is too large for this platform"))
}

impl DirectXAtlas {
    pub(crate) fn new(device: &ID3D11Device, device_context: &ID3D11DeviceContext) -> Self {
        DirectXAtlas(Mutex::new(DirectXAtlasState {
            device: device.clone(),
            device_context: device_context.clone(),
            monochrome_textures: Default::default(),
            polychrome_textures: Default::default(),
            video_textures: Vec::new(),
            video_free_list: Vec::new(),
            tiles_by_key: Default::default(),
        }))
    }

    pub(crate) fn get_texture_view(
        &self,
        id: AtlasTextureId,
    ) -> [Option<ID3D11ShaderResourceView>; 1] {
        let lock = self.0.lock();
        let tex = lock.texture(id);
        tex.view.clone()
    }

    pub(crate) fn get_video_texture_view(
        &self,
        id: VideoTextureId,
    ) -> Option<[Option<ID3D11ShaderResourceView>; 1]> {
        let lock = self.0.lock();
        lock.video_texture(id).map(|texture| texture.view.clone())
    }

    pub(crate) fn handle_device_lost(
        &self,
        device: &ID3D11Device,
        device_context: &ID3D11DeviceContext,
    ) {
        let mut lock = self.0.lock();
        lock.device = device.clone();
        lock.device_context = device_context.clone();
        lock.monochrome_textures = AtlasTextureList::default();
        lock.polychrome_textures = AtlasTextureList::default();
        lock.tiles_by_key.clear();

        let device = lock.device.clone();
        let mut failed_recreations = Vec::new();
        for (index, slot) in lock.video_textures.iter_mut().enumerate() {
            let Some(texture) = slot.texture.as_ref() else {
                continue;
            };
            match DirectXVideoTexture::new(&device, texture.width, texture.height) {
                Ok(new_texture) => {
                    slot.texture = Some(new_texture);
                }
                Err(error) => {
                    log::error!(
                        "failed to recreate DirectX video texture {} after device loss: {error:#}",
                        index
                    );
                    slot.texture = None;
                    failed_recreations.push(index);
                }
            }
        }
        for index in failed_recreations {
            if !lock.video_free_list.contains(&index) {
                lock.video_free_list.push(index);
            }
        }
    }
}

impl PlatformAtlas for DirectXAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> anyhow::Result<
            Option<(Size<DevicePixels>, std::borrow::Cow<'a, [u8]>)>,
        >,
    ) -> anyhow::Result<Option<AtlasTile>> {
        let mut lock = self.0.lock();
        if let Some(tile) = lock.tiles_by_key.get(key) {
            Ok(Some(tile.clone()))
        } else {
            let Some((size, bytes)) = build()? else {
                return Ok(None);
            };
            let tile = lock
                .allocate(size, key.texture_kind())
                .ok_or_else(|| anyhow::anyhow!("failed to allocate"))?;
            let texture = lock.texture(tile.texture_id);
            texture.upload(&lock.device_context, tile.bounds, &bytes);
            lock.tiles_by_key.insert(key.clone(), tile.clone());
            Ok(Some(tile))
        }
    }

    fn remove(&self, key: &AtlasKey) {
        let mut lock = self.0.lock();

        let Some(id) = lock.tiles_by_key.remove(key).map(|tile| tile.texture_id) else {
            return;
        };

        let textures = match id.kind {
            AtlasTextureKind::Monochrome => &mut lock.monochrome_textures,
            AtlasTextureKind::Polychrome => &mut lock.polychrome_textures,
        };

        let Some(texture_slot) = textures.textures.get_mut(id.index as usize) else {
            return;
        };

        if let Some(mut texture) = texture_slot.take() {
            texture.decrement_ref_count();
            if texture.is_unreferenced() {
                textures.free_list.push(texture.id.index as usize);
                lock.tiles_by_key.remove(key);
            } else {
                *texture_slot = Some(texture);
            }
        }
    }

    fn alloc_video_texture(&self, width: u32, height: u32) -> Result<VideoTextureId> {
        video_frame_byte_len(width, height)?;

        let mut lock = self.0.lock();
        let texture = DirectXVideoTexture::new(&lock.device, width, height)?;
        let (index, generation) = if let Some(i) = lock.video_free_list.pop() {
            let slot = &mut lock.video_textures[i];
            slot.generation = next_video_generation(slot.generation);
            slot.texture = Some(texture);
            (i, slot.generation)
        } else {
            let i = lock.video_textures.len();
            let generation = 1;
            lock.video_textures.push(DirectXVideoTextureSlot {
                generation,
                texture: Some(texture),
            });
            (i, generation)
        };

        let index = u32::try_from(index).map_err(|_| anyhow!("too many video textures"))?;
        Ok(VideoTextureId::new(index, generation))
    }

    fn upload_video_frame(&self, id: VideoTextureId, data: &[u8]) -> Result<()> {
        let lock = self.0.lock();
        let Some(texture) = lock.video_texture(id) else {
            bail!("VideoTextureId refers to a freed or never-allocated texture");
        };

        let expected_len = video_frame_byte_len(texture.width, texture.height)?;
        if data.len() != expected_len {
            bail!(
                "invalid video frame byte length: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        texture.upload(&lock.device_context, data)?;
        Ok(())
    }

    fn free_video_texture(&self, id: VideoTextureId) {
        let mut lock = self.0.lock();
        let index = id.index as usize;
        let mut freed = false;
        if let Some(slot) = lock.video_textures.get_mut(index) {
            if slot.generation == id.generation {
                freed = slot.texture.take().is_some();
            }
        }

        if freed && !lock.video_free_list.contains(&index) {
            lock.video_free_list.push(index);
        }
    }
}

impl DirectXAtlasState {
    fn video_texture(&self, id: VideoTextureId) -> Option<&DirectXVideoTexture> {
        self.video_textures
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.texture.as_ref())
    }

    fn allocate(
        &mut self,
        size: Size<DevicePixels>,
        texture_kind: AtlasTextureKind,
    ) -> Option<AtlasTile> {
        {
            let textures = match texture_kind {
                AtlasTextureKind::Monochrome => &mut self.monochrome_textures,
                AtlasTextureKind::Polychrome => &mut self.polychrome_textures,
            };

            if let Some(tile) = textures
                .iter_mut()
                .rev()
                .find_map(|texture| texture.allocate(size))
            {
                return Some(tile);
            }
        }

        let texture = self.push_texture(size, texture_kind)?;
        texture.allocate(size)
    }

    fn push_texture(
        &mut self,
        min_size: Size<DevicePixels>,
        kind: AtlasTextureKind,
    ) -> Option<&mut DirectXAtlasTexture> {
        const DEFAULT_ATLAS_SIZE: Size<DevicePixels> = Size {
            width: DevicePixels(1024),
            height: DevicePixels(1024),
        };
        // Max texture size for DirectX. See:
        // https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-resources-limits
        const MAX_ATLAS_SIZE: Size<DevicePixels> = Size {
            width: DevicePixels(16384),
            height: DevicePixels(16384),
        };
        let size = min_size.min(&MAX_ATLAS_SIZE).max(&DEFAULT_ATLAS_SIZE);
        let pixel_format;
        let bind_flag;
        let bytes_per_pixel;
        match kind {
            AtlasTextureKind::Monochrome => {
                pixel_format = DXGI_FORMAT_R8_UNORM;
                bind_flag = D3D11_BIND_SHADER_RESOURCE;
                bytes_per_pixel = 1;
            }
            AtlasTextureKind::Polychrome => {
                pixel_format = DXGI_FORMAT_B8G8R8A8_UNORM;
                bind_flag = D3D11_BIND_SHADER_RESOURCE;
                bytes_per_pixel = 4;
            }
        }
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: size.width.0 as u32,
            Height: size.height.0 as u32,
            MipLevels: 1,
            ArraySize: 1,
            Format: pixel_format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: bind_flag.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        unsafe {
            // This only returns None if the device is lost, which we will recreate later.
            // So it's ok to return None here.
            self.device
                .CreateTexture2D(&texture_desc, None, Some(&mut texture))
                .ok()?;
        }
        let texture = texture.unwrap();

        let texture_list = match kind {
            AtlasTextureKind::Monochrome => &mut self.monochrome_textures,
            AtlasTextureKind::Polychrome => &mut self.polychrome_textures,
        };
        let index = texture_list.free_list.pop();
        let view = unsafe {
            let mut view = None;
            self.device
                .CreateShaderResourceView(&texture, None, Some(&mut view))
                .ok()?;
            [view]
        };
        let atlas_texture = DirectXAtlasTexture {
            id: AtlasTextureId {
                index: index.unwrap_or(texture_list.textures.len()) as u32,
                kind,
            },
            bytes_per_pixel,
            allocator: etagere::BucketedAtlasAllocator::new(size.into()),
            texture,
            view,
            live_atlas_keys: 0,
        };
        if let Some(ix) = index {
            texture_list.textures[ix] = Some(atlas_texture);
            texture_list.textures.get_mut(ix).unwrap().as_mut()
        } else {
            texture_list.textures.push(Some(atlas_texture));
            texture_list.textures.last_mut().unwrap().as_mut()
        }
    }

    fn texture(&self, id: AtlasTextureId) -> &DirectXAtlasTexture {
        let textures = match id.kind {
            crate::AtlasTextureKind::Monochrome => &self.monochrome_textures,
            crate::AtlasTextureKind::Polychrome => &self.polychrome_textures,
        };
        textures[id.index as usize].as_ref().unwrap()
    }
}

impl DirectXAtlasTexture {
    fn allocate(&mut self, size: Size<DevicePixels>) -> Option<AtlasTile> {
        let allocation = self.allocator.allocate(size.into())?;
        let tile = AtlasTile {
            texture_id: self.id,
            tile_id: allocation.id.into(),
            bounds: Bounds {
                origin: allocation.rectangle.min.into(),
                size,
            },
            padding: 0,
        };
        self.live_atlas_keys += 1;
        Some(tile)
    }

    fn upload(
        &self,
        device_context: &ID3D11DeviceContext,
        bounds: Bounds<DevicePixels>,
        bytes: &[u8],
    ) {
        unsafe {
            device_context.UpdateSubresource(
                &self.texture,
                0,
                Some(&D3D11_BOX {
                    left: bounds.left().0 as u32,
                    top: bounds.top().0 as u32,
                    front: 0,
                    right: bounds.right().0 as u32,
                    bottom: bounds.bottom().0 as u32,
                    back: 1,
                }),
                bytes.as_ptr() as _,
                bounds.size.width.to_bytes(self.bytes_per_pixel as u8),
                0,
            );
        }
    }

    fn decrement_ref_count(&mut self) {
        self.live_atlas_keys -= 1;
    }

    fn is_unreferenced(&mut self) -> bool {
        self.live_atlas_keys == 0
    }
}

impl DirectXVideoTexture {
    fn new(device: &ID3D11Device, width: u32, height: u32) -> Result<Self> {
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut texture = None;
        unsafe {
            device.CreateTexture2D(&texture_desc, None, Some(&mut texture))?;
        }
        let texture =
            texture.ok_or_else(|| anyhow!("DirectX returned no video texture allocation"))?;

        let view = unsafe {
            let mut view = None;
            device.CreateShaderResourceView(&texture, None, Some(&mut view))?;
            [view]
        };

        Ok(Self {
            width,
            height,
            texture,
            view,
        })
    }

    fn upload(&self, device_context: &ID3D11DeviceContext, data: &[u8]) -> Result<()> {
        let row_pitch = self
            .width
            .checked_mul(4)
            .ok_or_else(|| anyhow!("video frame row pitch overflows u32"))?;
        unsafe {
            device_context.UpdateSubresource(
                &self.texture,
                0,
                None,
                data.as_ptr() as _,
                row_pitch,
                0,
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_frame_byte_len_requires_non_zero_dimensions() {
        assert!(video_frame_byte_len(0, 1).is_err());
        assert!(video_frame_byte_len(1, 0).is_err());
    }

    #[test]
    fn video_frame_byte_len_matches_rgba8_extent() {
        assert_eq!(video_frame_byte_len(16, 9).unwrap(), 16 * 9 * 4);
    }

    #[test]
    fn video_frame_byte_len_rejects_dimensions_above_directx_limit() {
        assert!(video_frame_byte_len(16385, 1).is_err());
        assert!(video_frame_byte_len(1, 16385).is_err());
    }

    #[test]
    fn video_texture_generations_skip_zero_after_wrapping() {
        assert_eq!(next_video_generation(u32::MAX), 1);
    }
}

impl From<Size<DevicePixels>> for etagere::Size {
    fn from(size: Size<DevicePixels>) -> Self {
        etagere::Size::new(size.width.into(), size.height.into())
    }
}

impl From<etagere::Point> for Point<DevicePixels> {
    fn from(value: etagere::Point) -> Self {
        Point {
            x: DevicePixels::from(value.x),
            y: DevicePixels::from(value.y),
        }
    }
}
