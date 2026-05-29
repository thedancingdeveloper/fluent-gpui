use crate::{
    AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, Bounds, DevicePixels, PlatformAtlas,
    Point, Size, VideoTextureId, platform::AtlasTextureList,
};
use anyhow::{Result, anyhow, bail};
use blade_graphics as gpu;
use blade_util::{BufferBelt, BufferBeltDescriptor};
use collections::FxHashMap;
use etagere::BucketedAtlasAllocator;
use parking_lot::Mutex;
use std::{borrow::Cow, ops, sync::Arc};

pub(crate) struct BladeAtlas(Mutex<BladeAtlasState>);

struct PendingUpload {
    id: AtlasTextureId,
    bounds: Bounds<DevicePixels>,
    data: gpu::BufferPiece,
}

struct PendingVideoUpload {
    id: VideoTextureId,
    data: Vec<u8>,
}

struct VideoTexture {
    raw: gpu::Texture,
    raw_view: gpu::TextureView,
    width: u32,
    height: u32,
}

struct VideoTextureSlot {
    generation: u32,
    texture: Option<VideoTexture>,
}

fn next_video_generation(generation: u32) -> u32 {
    let next = generation.wrapping_add(1);
    if next == 0 { 1 } else { next }
}

fn video_frame_byte_len(width: u32, height: u32) -> Result<usize> {
    if width == 0 || height == 0 {
        bail!("video texture dimensions must be non-zero");
    }

    let byte_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|len| len.checked_mul(4))
        .ok_or_else(|| anyhow!("video frame dimensions overflow byte length"))?;

    usize::try_from(byte_len).map_err(|_| anyhow!("video frame is too large for this platform"))
}

struct BladeAtlasState {
    gpu: Arc<gpu::Context>,
    upload_belt: BufferBelt,
    storage: BladeAtlasStorage,
    tiles_by_key: FxHashMap<AtlasKey, AtlasTile>,
    initializations: Vec<AtlasTextureId>,
    uploads: Vec<PendingUpload>,
    video_textures: Vec<VideoTextureSlot>,
    video_free_list: Vec<usize>,
    video_uploads: Vec<PendingVideoUpload>,
    video_initializations: Vec<VideoTextureId>,
}

#[cfg(gles)]
unsafe impl Send for BladeAtlasState {}

impl BladeAtlasState {
    fn destroy(&mut self) {
        self.storage.destroy(&self.gpu);
        for slot in self.video_textures.drain(..) {
            if let Some(mut texture) = slot.texture {
                texture.destroy(&self.gpu);
            }
        }
        self.upload_belt.destroy(&self.gpu);
    }
}

pub struct BladeTextureInfo {
    pub raw_view: gpu::TextureView,
}

impl BladeAtlas {
    pub(crate) fn new(gpu: &Arc<gpu::Context>) -> Self {
        BladeAtlas(Mutex::new(BladeAtlasState {
            gpu: Arc::clone(gpu),
            upload_belt: BufferBelt::new(BufferBeltDescriptor {
                memory: gpu::Memory::Upload,
                min_chunk_size: 0x10000,
                alignment: 64, // Vulkan `optimalBufferCopyOffsetAlignment` on Intel XE
            }),
            storage: BladeAtlasStorage::default(),
            tiles_by_key: Default::default(),
            initializations: Vec::new(),
            uploads: Vec::new(),
            video_textures: Vec::new(),
            video_free_list: Vec::new(),
            video_uploads: Vec::new(),
            video_initializations: Vec::new(),
        }))
    }

    pub(crate) fn destroy(&self) {
        self.0.lock().destroy();
    }

    pub fn before_frame(&self, gpu_encoder: &mut gpu::CommandEncoder) {
        let mut lock = self.0.lock();
        lock.flush(gpu_encoder);
    }

    pub fn after_frame(&self, sync_point: &gpu::SyncPoint) {
        let mut lock = self.0.lock();
        lock.upload_belt.flush(sync_point);
    }

    pub fn get_texture_info(&self, id: AtlasTextureId) -> BladeTextureInfo {
        let lock = self.0.lock();
        let texture = &lock.storage[id];
        BladeTextureInfo {
            raw_view: texture.raw_view,
        }
    }

    pub fn get_video_texture_view(&self, id: VideoTextureId) -> Option<gpu::TextureView> {
        let lock = self.0.lock();
        lock.video_texture(id).map(|texture| texture.raw_view)
    }
}

impl PlatformAtlas for BladeAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> Result<Option<AtlasTile>> {
        let mut lock = self.0.lock();
        if let Some(tile) = lock.tiles_by_key.get(key) {
            Ok(Some(tile.clone()))
        } else {
            profiling::scope!("new tile");
            let Some((size, bytes)) = build()? else {
                return Ok(None);
            };
            let tile = lock.allocate(size, key.texture_kind());
            lock.upload_texture(tile.texture_id, tile.bounds, &bytes);
            lock.tiles_by_key.insert(key.clone(), tile.clone());
            Ok(Some(tile))
        }
    }

    fn remove(&self, key: &AtlasKey) {
        let mut lock = self.0.lock();

        let Some(id) = lock.tiles_by_key.remove(key).map(|tile| tile.texture_id) else {
            return;
        };

        let Some(texture_slot) = lock.storage[id.kind].textures.get_mut(id.index as usize) else {
            return;
        };

        if let Some(mut texture) = texture_slot.take() {
            texture.decrement_ref_count();
            if texture.is_unreferenced() {
                lock.storage[id.kind]
                    .free_list
                    .push(texture.id.index as usize);
                texture.destroy(&lock.gpu);
            } else {
                *texture_slot = Some(texture);
            }
        }
    }

    fn alloc_video_texture(&self, width: u32, height: u32) -> Result<VideoTextureId> {
        video_frame_byte_len(width, height)?;

        let mut lock = self.0.lock();
        let gpu = Arc::clone(&lock.gpu);
        let raw = gpu.create_texture(gpu::TextureDesc {
            name: "video-frame",
            format: gpu::TextureFormat::Rgba8Unorm,
            size: gpu::Extent {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: gpu::TextureDimension::D2,
            usage: gpu::TextureUsage::COPY | gpu::TextureUsage::RESOURCE,
            external: None,
        });
        let raw_view = gpu.create_texture_view(
            raw,
            gpu::TextureViewDesc {
                name: "video-frame-view",
                format: gpu::TextureFormat::Rgba8Unorm,
                dimension: gpu::ViewDimension::D2,
                subresources: &Default::default(),
            },
        );
        let (index, generation) = if let Some(i) = lock.video_free_list.pop() {
            let slot = &mut lock.video_textures[i];
            slot.generation = next_video_generation(slot.generation);
            slot.texture = Some(VideoTexture {
                raw,
                raw_view,
                width,
                height,
            });
            (i, slot.generation)
        } else {
            let i = lock.video_textures.len();
            let generation = 1;
            lock.video_textures.push(VideoTextureSlot {
                generation,
                texture: Some(VideoTexture {
                    raw,
                    raw_view,
                    width,
                    height,
                }),
            });
            (i, generation)
        };
        let index = u32::try_from(index).map_err(|_| anyhow!("too many video textures"))?;
        let id = VideoTextureId::new(index, generation);
        lock.video_initializations.push(id);
        Ok(id)
    }

    fn upload_video_frame(&self, id: VideoTextureId, data: &[u8]) -> Result<()> {
        let mut lock = self.0.lock();
        let Some((width, height)) = lock
            .video_texture(id)
            .map(|texture| (texture.width, texture.height))
        else {
            bail!("VideoTextureId refers to a freed or never-allocated texture");
        };

        let expected_len = video_frame_byte_len(width, height)?;
        if data.len() != expected_len {
            bail!(
                "invalid video frame byte length: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        if let Some(upload) = lock.video_uploads.iter_mut().find(|upload| upload.id == id) {
            upload.data.clear();
            upload.data.extend_from_slice(data);
        } else {
            lock.video_uploads.push(PendingVideoUpload {
                id,
                data: data.to_vec(),
            });
        }

        Ok(())
    }

    fn free_video_texture(&self, id: VideoTextureId) {
        let mut lock = self.0.lock();
        let index = id.index as usize;
        let mut texture_to_destroy = None;
        if let Some(slot) = lock.video_textures.get_mut(index) {
            if slot.generation == id.generation {
                texture_to_destroy = slot.texture.take();
            }
        }

        if let Some(mut texture) = texture_to_destroy {
            lock.video_initializations
                .retain(|initialization| *initialization != id);
            lock.video_uploads.retain(|upload| upload.id != id);
            if !lock.video_free_list.contains(&index) {
                lock.video_free_list.push(index);
            }
            texture.destroy(&lock.gpu);
        }
    }
}

impl BladeAtlasState {
    fn video_texture(&self, id: VideoTextureId) -> Option<&VideoTexture> {
        self.video_textures
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.texture.as_ref())
    }

    fn allocate(&mut self, size: Size<DevicePixels>, texture_kind: AtlasTextureKind) -> AtlasTile {
        {
            let textures = &mut self.storage[texture_kind];

            if let Some(tile) = textures
                .iter_mut()
                .rev()
                .find_map(|texture| texture.allocate(size))
            {
                return tile;
            }
        }

        let texture = self.push_texture(size, texture_kind);
        texture.allocate(size).unwrap()
    }

    fn push_texture(
        &mut self,
        min_size: Size<DevicePixels>,
        kind: AtlasTextureKind,
    ) -> &mut BladeAtlasTexture {
        const DEFAULT_ATLAS_SIZE: Size<DevicePixels> = Size {
            width: DevicePixels(1024),
            height: DevicePixels(1024),
        };

        let size = min_size.max(&DEFAULT_ATLAS_SIZE);
        let format;
        let usage;
        match kind {
            AtlasTextureKind::Monochrome => {
                format = gpu::TextureFormat::R8Unorm;
                usage = gpu::TextureUsage::COPY | gpu::TextureUsage::RESOURCE;
            }
            AtlasTextureKind::Polychrome => {
                format = gpu::TextureFormat::Bgra8Unorm;
                usage = gpu::TextureUsage::COPY | gpu::TextureUsage::RESOURCE;
            }
        }

        let raw = self.gpu.create_texture(gpu::TextureDesc {
            name: "atlas",
            format,
            size: gpu::Extent {
                width: size.width.into(),
                height: size.height.into(),
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: gpu::TextureDimension::D2,
            usage,
            external: None,
        });
        let raw_view = self.gpu.create_texture_view(
            raw,
            gpu::TextureViewDesc {
                name: "",
                format,
                dimension: gpu::ViewDimension::D2,
                subresources: &Default::default(),
            },
        );

        let texture_list = &mut self.storage[kind];
        let index = texture_list.free_list.pop();

        let atlas_texture = BladeAtlasTexture {
            id: AtlasTextureId {
                index: index.unwrap_or(texture_list.textures.len()) as u32,
                kind,
            },
            allocator: etagere::BucketedAtlasAllocator::new(size.into()),
            format,
            raw,
            raw_view,
            live_atlas_keys: 0,
        };

        self.initializations.push(atlas_texture.id);

        if let Some(ix) = index {
            texture_list.textures[ix] = Some(atlas_texture);
            texture_list.textures.get_mut(ix).unwrap().as_mut().unwrap()
        } else {
            texture_list.textures.push(Some(atlas_texture));
            texture_list.textures.last_mut().unwrap().as_mut().unwrap()
        }
    }

    fn upload_texture(&mut self, id: AtlasTextureId, bounds: Bounds<DevicePixels>, bytes: &[u8]) {
        let data = self.upload_belt.alloc_bytes(bytes, &self.gpu);
        self.uploads.push(PendingUpload { id, bounds, data });
    }

    fn flush_initializations(&mut self, encoder: &mut gpu::CommandEncoder) {
        for id in self.initializations.drain(..) {
            let texture = &self.storage[id];
            encoder.init_texture(texture.raw);
        }
        let video_initializations = std::mem::take(&mut self.video_initializations);
        for id in video_initializations {
            if let Some(vt) = self.video_texture(id) {
                encoder.init_texture(vt.raw);
            }
        }
    }

    fn flush(&mut self, encoder: &mut gpu::CommandEncoder) {
        self.flush_initializations(encoder);

        let mut transfers = encoder.transfer("atlas");
        for upload in self.uploads.drain(..) {
            let texture = &self.storage[upload.id];
            transfers.copy_buffer_to_texture(
                upload.data,
                upload.bounds.size.width.to_bytes(texture.bytes_per_pixel()),
                gpu::TexturePiece {
                    texture: texture.raw,
                    mip_level: 0,
                    array_layer: 0,
                    origin: [
                        upload.bounds.origin.x.into(),
                        upload.bounds.origin.y.into(),
                        0,
                    ],
                },
                gpu::Extent {
                    width: upload.bounds.size.width.into(),
                    height: upload.bounds.size.height.into(),
                    depth: 1,
                },
            );
        }
        let video_uploads = std::mem::take(&mut self.video_uploads);
        for upload in video_uploads {
            let Some((width, height, raw)) = self
                .video_texture(upload.id)
                .map(|texture| (texture.width, texture.height, texture.raw))
            else {
                continue;
            };
            let buf = self.upload_belt.alloc_bytes(&upload.data, &self.gpu);
            transfers.copy_buffer_to_texture(
                buf,
                width * 4,
                gpu::TexturePiece {
                    texture: raw,
                    mip_level: 0,
                    array_layer: 0,
                    origin: [0, 0, 0],
                },
                gpu::Extent {
                    width,
                    height,
                    depth: 1,
                },
            );
        }
    }
}

#[derive(Default)]
struct BladeAtlasStorage {
    monochrome_textures: AtlasTextureList<BladeAtlasTexture>,
    polychrome_textures: AtlasTextureList<BladeAtlasTexture>,
}

impl ops::Index<AtlasTextureKind> for BladeAtlasStorage {
    type Output = AtlasTextureList<BladeAtlasTexture>;
    fn index(&self, kind: AtlasTextureKind) -> &Self::Output {
        match kind {
            crate::AtlasTextureKind::Monochrome => &self.monochrome_textures,
            crate::AtlasTextureKind::Polychrome => &self.polychrome_textures,
        }
    }
}

impl ops::IndexMut<AtlasTextureKind> for BladeAtlasStorage {
    fn index_mut(&mut self, kind: AtlasTextureKind) -> &mut Self::Output {
        match kind {
            crate::AtlasTextureKind::Monochrome => &mut self.monochrome_textures,
            crate::AtlasTextureKind::Polychrome => &mut self.polychrome_textures,
        }
    }
}

impl ops::Index<AtlasTextureId> for BladeAtlasStorage {
    type Output = BladeAtlasTexture;
    fn index(&self, id: AtlasTextureId) -> &Self::Output {
        let textures = match id.kind {
            crate::AtlasTextureKind::Monochrome => &self.monochrome_textures,
            crate::AtlasTextureKind::Polychrome => &self.polychrome_textures,
        };
        textures[id.index as usize].as_ref().unwrap()
    }
}

impl BladeAtlasStorage {
    fn destroy(&mut self, gpu: &gpu::Context) {
        for mut texture in self.monochrome_textures.drain().flatten() {
            texture.destroy(gpu);
        }
        for mut texture in self.polychrome_textures.drain().flatten() {
            texture.destroy(gpu);
        }
    }
}

struct BladeAtlasTexture {
    id: AtlasTextureId,
    allocator: BucketedAtlasAllocator,
    raw: gpu::Texture,
    raw_view: gpu::TextureView,
    format: gpu::TextureFormat,
    live_atlas_keys: u32,
}

impl BladeAtlasTexture {
    fn allocate(&mut self, size: Size<DevicePixels>) -> Option<AtlasTile> {
        let allocation = self.allocator.allocate(size.into())?;
        let tile = AtlasTile {
            texture_id: self.id,
            tile_id: allocation.id.into(),
            padding: 0,
            bounds: Bounds {
                origin: allocation.rectangle.min.into(),
                size,
            },
        };
        self.live_atlas_keys += 1;
        Some(tile)
    }

    fn destroy(&mut self, gpu: &gpu::Context) {
        gpu.destroy_texture(self.raw);
        gpu.destroy_texture_view(self.raw_view);
    }

    fn bytes_per_pixel(&self) -> u8 {
        self.format.block_info().size
    }

    fn decrement_ref_count(&mut self) {
        self.live_atlas_keys -= 1;
    }

    fn is_unreferenced(&mut self) -> bool {
        self.live_atlas_keys == 0
    }
}

impl VideoTexture {
    fn destroy(&mut self, gpu: &gpu::Context) {
        gpu.destroy_texture(self.raw);
        gpu.destroy_texture_view(self.raw_view);
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

impl From<etagere::Size> for Size<DevicePixels> {
    fn from(size: etagere::Size) -> Self {
        Size {
            width: DevicePixels::from(size.width),
            height: DevicePixels::from(size.height),
        }
    }
}

impl From<etagere::Rectangle> for Bounds<DevicePixels> {
    fn from(rectangle: etagere::Rectangle) -> Self {
        Bounds {
            origin: rectangle.min.into(),
            size: rectangle.size().into(),
        }
    }
}
