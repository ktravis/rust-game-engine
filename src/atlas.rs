use image::{GenericImage, RgbaImage};

use crate::geom::{Point, Rect};

#[derive(Debug)]
pub struct Atlas {
    image: RgbaImage,
    entries: Vec<AtlasRegion>,
}

impl Atlas {
    const DEFAULT_SIZE: u32 = 256;

    pub fn size(&self) -> Point<u32> {
        Point::new(self.image.width(), self.image.height())
    }

    pub fn image(&self) -> &RgbaImage {
        &self.image
    }

    pub fn entry_rect(&self, i: usize) -> Rect {
        self.normalized_region(&self.entries[i])
    }

    fn normalized_region(&self, region: &AtlasRegion) -> Rect {
        let w = self.image.width();
        let h = self.image.height();
        Rect {
            pos: glam::vec2(
                region.pos.x as f32 / w as f32,
                region.pos.y as f32 / h as f32,
            ),
            dim: glam::vec2(
                region.dim.x as f32 / w as f32,
                region.dim.y as f32 / h as f32,
            ),
        }
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            image: RgbaImage::from_pixel(
                Self::DEFAULT_SIZE,
                Self::DEFAULT_SIZE,
                [255, 0, 255, 255].into(),
            ),
        }
    }
}

#[derive(Default, Debug)]
pub struct AtlasBuilder {
    extents: Point<u32>,
    cursor: Point<u32>,
    atlas: Atlas,
}

impl AtlasBuilder {
    pub fn from_images(images: &[RgbaImage]) -> image::ImageResult<Self> {
        let mut b = Self::default();
        for img in images {
            b.add(img)?;
        }
        Ok(b)
    }

    pub fn add(&mut self, img: &RgbaImage) -> image::ImageResult<()> {
        if self.cursor.x + img.width() > self.atlas.image.width() {
            // go to the next available row
            self.cursor.x = 0;
            self.cursor.y = self.extents.y;
        }
        if self.cursor.y + img.height() > self.atlas.image.height()
            || self.cursor.x + img.width() > self.atlas.image.width()
        {
            // atlas is full, resize
            self.resize(self.atlas.size() * 2)?;
            self.cursor.x = self.extents.x;
            self.cursor.y = 0;
            return self.add(img);
        }
        let dim = Point::new(img.width(), img.height());
        let pos = self.cursor;
        self.atlas.image.copy_from(img, pos.x, pos.y)?;
        self.atlas.entries.push(AtlasRegion { pos, dim });
        self.cursor.x += img.width();
        self.extents.x = self.extents.x.max(self.cursor.x);
        self.extents.y = self.extents.y.max(self.cursor.y + img.height());
        Ok(())
    }

    fn resize(&mut self, new_size: Point<u32>) -> image::ImageResult<()> {
        let old_atlas = &self.atlas.image;
        println!("resizing atlas: ({}, {})", new_size.x, new_size.y);
        let mut a = RgbaImage::from_pixel(new_size.x, new_size.y, [255, 0, 255, 255].into());
        a.copy_from(old_atlas, 0, 0)?;
        self.atlas.image = a;
        Ok(())
    }

    pub fn build(self) -> Atlas {
        self.atlas
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub pos: Point<u32>,
    pub dim: Point<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_builder() {
        let items = [
            RgbaImage::from_pixel(12, 12, [255, 0, 0, 255].into()),
            RgbaImage::from_pixel(24, 12, [255, 255, 0, 255].into()),
            RgbaImage::from_pixel(12, 56, [255, 255, 255, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 12, [0, 0, 255, 255].into()),
            RgbaImage::from_pixel(12, 8, [0, 255, 0, 255].into()),
            RgbaImage::from_pixel(24, 2, [128, 255, 0, 255].into()),
            RgbaImage::from_pixel(74, 50, [55, 128, 55, 255].into()),
            RgbaImage::from_pixel(50, 74, [55, 128, 100, 255].into()),
            RgbaImage::from_pixel(74, 50, [100, 128, 100, 255].into()),
            RgbaImage::from_pixel(74, 50, [55, 128, 55, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 56, [255, 255, 255, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 12, [0, 0, 255, 255].into()),
            RgbaImage::from_pixel(12, 8, [0, 255, 0, 255].into()),
            RgbaImage::from_pixel(12, 8, [0, 255, 0, 255].into()),
            RgbaImage::from_pixel(24, 2, [128, 255, 0, 255].into()),
            RgbaImage::from_pixel(74, 50, [55, 128, 55, 255].into()),
            RgbaImage::from_pixel(50, 74, [55, 128, 100, 255].into()),
            RgbaImage::from_pixel(74, 50, [100, 128, 100, 255].into()),
            RgbaImage::from_pixel(74, 50, [55, 128, 55, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 56, [255, 255, 255, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 56, [255, 255, 255, 255].into()),
            RgbaImage::from_pixel(100, 100, [255, 128, 0, 255].into()),
            RgbaImage::from_pixel(12, 12, [0, 0, 255, 255].into()),
            RgbaImage::from_pixel(24, 12, [255, 255, 0, 255].into()),
            RgbaImage::from_pixel(50, 74, [55, 128, 100, 255].into()),
            RgbaImage::from_pixel(24, 2, [128, 255, 0, 255].into()),
            RgbaImage::from_pixel(74, 50, [55, 128, 55, 255].into()),
            RgbaImage::from_pixel(12, 8, [0, 255, 0, 255].into()),
        ];

        let atlas = AtlasBuilder::from_images(items.as_slice()).unwrap().build();
        atlas
            .image()
            .save(std::path::Path::new("testdata/final.png"))
            .unwrap();
    }
}
