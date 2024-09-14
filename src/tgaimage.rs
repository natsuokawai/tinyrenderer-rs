use std::fs::File;
use std::io::{self, Read, Write};

#[repr(C, packed)]
struct TGAHeader {
    idlength: u8,
    colormaptype: u8,
    datatypecode: u8,
    colormaporigin: u16,
    colormaplength: u16,
    colormapdepth: u8,
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bitsperpixel: u8,
    imagedescriptor: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct TGAColor {
    pub raw: [u8; 4],
    #[allow(dead_code)]
    pub bytespp: usize,
}

impl TGAColor {
    #[allow(dead_code)]
    pub fn new() -> TGAColor {
        TGAColor {
            raw: [0, 0, 0, 0],
            bytespp: 1,
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> TGAColor {
        TGAColor {
            raw: [b, g, r, a],
            bytespp: 4,
        }
    }

    #[allow(dead_code)]
    pub fn from_slice(p: &[u8], bpp: usize) -> TGAColor {
        let mut raw = [0u8; 4];
        raw[..bpp].copy_from_slice(&p[..bpp]);
        TGAColor { raw, bytespp: bpp }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Format {
    #[allow(dead_code)]
    Grayscale = 1,
    RGB = 3,
    #[allow(dead_code)]
    RGBA = 4,
}

pub struct TGAImage {
    pub data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub bytespp: usize,
}

impl TGAImage {
    pub fn new(w: i32, h: i32, format: Format) -> TGAImage {
        let bytespp = format as usize;
        let nbytes = (w * h * bytespp as i32) as usize;
        TGAImage {
            data: vec![0; nbytes],
            width: w,
            height: h,
            bytespp,
        }
    }

    #[allow(dead_code)]
    pub fn read_tga_file(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let mut header = TGAHeader {
            idlength: 0,
            colormaptype: 0,
            datatypecode: 0,
            colormaporigin: 0,
            colormaplength: 0,
            colormapdepth: 0,
            x_origin: 0,
            y_origin: 0,
            width: 0,
            height: 0,
            bitsperpixel: 0,
            imagedescriptor: 0,
        };

        unsafe {
            let header_bytes = std::slice::from_raw_parts_mut(
                &mut header as *mut _ as *mut u8,
                std::mem::size_of::<TGAHeader>(),
            );
            file.read_exact(header_bytes)?;
        }

        self.width = header.width as i32;
        self.height = header.height as i32;
        self.bytespp = (header.bitsperpixel >> 3) as usize;

        if self.width <= 0 || self.height <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid image dimensions",
            ));
        }

        let nbytes = (self.width * self.height * self.bytespp as i32) as usize;
        self.data.resize(nbytes, 0);

        if header.datatypecode == 2 || header.datatypecode == 3 {
            file.read_exact(&mut self.data)?;
        } else if header.datatypecode == 10 || header.datatypecode == 11 {
            self.load_rle_data(&mut file)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown file format",
            ));
        }

        if (header.imagedescriptor & 0x20) == 0 {
            self.flip_vertically();
        }
        if (header.imagedescriptor & 0x10) != 0 {
            self.flip_horizontally();
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn load_rle_data(&mut self, file: &mut File) -> io::Result<()> {
        let pixelcount = (self.width * self.height) as usize;
        let mut currentpixel = 0;
        let mut currentbyte = 0;

        while currentpixel < pixelcount {
            let chunkheader = {
                let mut buf = [0u8; 1];
                file.read_exact(&mut buf)?;
                buf[0]
            };

            if chunkheader < 128 {
                let chunklength = chunkheader as usize + 1;
                for _ in 0..chunklength {
                    let mut buf = vec![0u8; self.bytespp];
                    file.read_exact(&mut buf)?;
                    let colorbuffer = TGAColor::from_slice(&buf, self.bytespp);
                    self.data[currentbyte..currentbyte + self.bytespp]
                        .copy_from_slice(&colorbuffer.raw[..self.bytespp]);
                    currentbyte += self.bytespp;
                    currentpixel += 1;
                }
            } else {
                let chunklength = (chunkheader - 127) as usize;
                let mut buf = vec![0u8; self.bytespp];
                file.read_exact(&mut buf)?;
                let colorbuffer = TGAColor::from_slice(&buf, self.bytespp);

                for _ in 0..chunklength {
                    self.data[currentbyte..currentbyte + self.bytespp]
                        .copy_from_slice(&colorbuffer.raw[..self.bytespp]);
                    currentbyte += self.bytespp;
                    currentpixel += 1;
                }
            }
        }

        Ok(())
    }

    pub fn write_tga_file(&self, filename: &str, rle: bool) -> io::Result<()> {
        let mut file = File::create(filename)?;

        let header = TGAHeader {
            idlength: 0,
            colormaptype: 0,
            datatypecode: if self.bytespp == 1 {
                if rle {
                    11
                } else {
                    3
                }
            } else {
                if rle {
                    10
                } else {
                    2
                }
            },
            colormaporigin: 0,
            colormaplength: 0,
            colormapdepth: 0,
            x_origin: 0,
            y_origin: 0,
            width: self.width as u16,
            height: self.height as u16,
            bitsperpixel: (self.bytespp * 8) as u8,
            imagedescriptor: 0x20,
        };

        unsafe {
            let header_bytes = std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<TGAHeader>(),
            );
            file.write_all(header_bytes)?;
        }

        if !rle {
            file.write_all(&self.data)?;
        } else {
            self.unload_rle_data(&mut file)?;
        }

        let developer_area_ref = [0u8; 4];
        let extension_area_ref = [0u8; 4];
        let footer = b"TRUEVISION-XFILE.\0";

        file.write_all(&developer_area_ref)?;
        file.write_all(&extension_area_ref)?;
        file.write_all(footer)?;

        Ok(())
    }

    fn unload_rle_data(&self, file: &mut File) -> io::Result<()> {
        const MAX_CHUNK_LENGTH: usize = 128;
        let npixels = (self.width * self.height) as usize;
        let mut curpix = 0;

        while curpix < npixels {
            let chunkstart = curpix * self.bytespp;
            let mut curbyte = chunkstart;
            let mut run_length = 1usize;
            let mut raw = true;

            while curpix + run_length < npixels && run_length < MAX_CHUNK_LENGTH {
                let mut succ_eq = true;
                for t in 0..self.bytespp {
                    if self.data[curbyte + t] != self.data[curbyte + t + self.bytespp] {
                        succ_eq = false;
                        break;
                    }
                }
                curbyte += self.bytespp;
                if run_length == 1 {
                    raw = !succ_eq;
                }
                if raw && succ_eq {
                    run_length -= 1;
                    break;
                }
                if !raw && !succ_eq {
                    break;
                }
                run_length += 1;
            }

            curpix += run_length;
            if raw {
                file.write_all(&[(run_length - 1) as u8])?;
                file.write_all(&self.data[chunkstart..chunkstart + run_length * self.bytespp])?;
            } else {
                file.write_all(&[(run_length + 127) as u8])?;
                file.write_all(&self.data[chunkstart..chunkstart + self.bytespp])?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, x: i32, y: i32) -> Option<TGAColor> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((x + y * self.width) * self.bytespp as i32) as usize;
        Some(TGAColor::from_slice(
            &self.data[idx..idx + self.bytespp],
            self.bytespp,
        ))
    }

    pub fn set(&mut self, x: i32, y: i32, c: &TGAColor) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return false;
        }
        let idx = ((x + y * self.width) * self.bytespp as i32) as usize;
        self.data[idx..idx + self.bytespp].copy_from_slice(&c.raw[..self.bytespp]);
        true
    }

    #[allow(dead_code)]
    pub fn flip_horizontally(&mut self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        let half = self.width / 2;
        for i in 0..half {
            for j in 0..self.height {
                let idx1 = ((i + j * self.width) * self.bytespp as i32) as usize;
                let idx2 = (((self.width - 1 - i) + j * self.width) * self.bytespp as i32) as usize;
                for b in 0..self.bytespp {
                    self.data.swap(idx1 + b, idx2 + b);
                }
            }
        }
        true
    }

    pub fn flip_vertically(&mut self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        let bytes_per_line = (self.width as usize) * self.bytespp;
        let half = (self.height / 2) as usize;
        let height_usize = self.height as usize;

        for j in 0..half {
            let idx1 = j * bytes_per_line;
            let idx2 = (height_usize - 1 - j) * bytes_per_line;

            let (top, rest) = self.data.split_at_mut(idx2);
            let line1 = &mut top[idx1..idx1 + bytes_per_line];
            let line2 = &mut rest[..bytes_per_line];

            line1.swap_with_slice(line2);
        }
        true
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    #[allow(dead_code)]
    pub fn scale(&mut self, w: i32, h: i32) -> bool {
        if w <= 0 || h <= 0 || self.data.is_empty() {
            return false;
        }
        let mut tdata = vec![0u8; (w * h * self.bytespp as i32) as usize];
        let mut nscanline = 0usize;
        let mut oscanline = 0usize;
        let mut erry = 0;
        let nlinebytes = (w as usize) * self.bytespp;
        let olinebytes = (self.width as usize) * self.bytespp;

        for _j in 0..self.height {
            let mut errx = self.width - w;
            let mut nx = -(self.bytespp as i32);
            let mut ox = -(self.bytespp as i32);
            for _i in 0..self.width {
                ox += self.bytespp as i32;
                errx += w;
                while errx >= self.width {
                    errx -= self.width;
                    nx += self.bytespp as i32;
                    tdata[(nscanline + nx as usize)..(nscanline + nx as usize + self.bytespp)]
                        .copy_from_slice(
                            &self.data[(oscanline + ox as usize)
                                ..(oscanline + ox as usize + self.bytespp)],
                        );
                }
            }
            erry += h;
            oscanline += olinebytes;
            while erry >= self.height {
                erry -= self.height;
                nscanline += nlinebytes;

                let dest_start = nscanline - nlinebytes;
                let dest_end = nscanline;

                let (before, after) = tdata.split_at_mut(dest_end);
                let dest = &mut before[dest_start..dest_end];
                let src = &after[0..nlinebytes];

                dest.copy_from_slice(src);
            }
        }

        self.data = tdata;
        self.width = w;
        self.height = h;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tgaimage_set() {
        let w = 2;
        let h = 2;
        let c = &TGAColor::rgba(128, 1, 255, 255);

        let mut image = TGAImage::new(w, h, Format::RGB);
        image.set(1, 1, c);
        image.flip_vertically();

        let mut testimage = TGAImage::new(w, h, Format::RGB);
        testimage.read_tga_file("tests/images/dot.tga").unwrap();

        assert_eq!(image.data, testimage.data);
    }
}
