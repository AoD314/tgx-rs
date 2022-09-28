use std::fs;
use std::env;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct Image {
    pos: u32,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl  Image {
    fn new(w: u32, h: u32) -> Self {
        Self {
            pos: 0,
            width: w,
            height: h,
            data: Vec::with_capacity((w * 3 * h) as usize),
        }
    }

    fn add_repeat(&mut self, count: u8, r: u8, g: u8, b: u8) {
        for _ in 0..count {
            self.data.push(b);
            self.data.push(g);
            self.data.push(r);
            self.pos += 1;
        }
    }

    fn add_newline(&mut self) {
        for _ in self.pos..self.width {
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
        }
        self.pos = 0;
    }

    fn write_to_bmp(&self, name: &str) {
        let mut contents: Vec<u8> = vec![
            // BMP header
            0x42, 0x4d, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 
            // DIB header (in BITMAPINFOHEADER format)
            0x28, 0x00, 0x00, 0x00, // The size of this header, in bytes; 0x28 = 40 bytes
            (self.width & 255) as u8, ((self.width >> 8) & 255) as u8, 0x00, 0x00, // The bitmap width in pixels
            (self.height & 255) as u8, ((self.height >> 8) & 255) as u8, 0x00, 0x00, // The bitmap height in pixels
            0x01, 0x00, // 2 bytes - The number of color planes
            0x18, 0x00, // 2 bytes - The number of bits per pixel
            0x00, 0x00, 0x00, 0x00, // The compression method being used: 0 = no compression
            0x00, 0x00, 0x00, 0x00, // The image size (in bytes). This is actually ignored for images without compression
            0x23 ,0x2e, 0x00, 0x00, // The horizontal resolution of the image
            0x23, 0x2e, 0x00, 0x00, // The vertical resolution of the image
            0x00, 0x00, 0x00, 0x00, // The number of colors in the color palette
            0x00, 0x00, 0x00, 0x00, // The number of important colors used, or 0 when every color is important
            // DATA
        ];

        let mut padding = (self.width * 3) % 4;
        if padding != 0 {
            padding = 4 - padding;
        }

        for j in 0..self.height {
            for i in 0..self.width {
                let index: usize = (self.width * 3 * j + 3 * i) as usize;
                contents.push(self.data[index+0]);
                contents.push(self.data[index+1]);
                contents.push(self.data[index+2]);
            }
            for _ in 0..padding {
                contents.push(0)
            }
        }

        fs::write(name, contents);
    }
}

fn parse_pixel(pixel: u16, r: &mut u8, g: &mut u8, b: &mut u8) {
    *r = (((pixel >> 1) & 0b1111) << 3) as u8;
    *g = (((pixel >> 6) & 0b1111) << 3) as u8;
    *r = (((pixel >> 11) & 0b1111) << 3) as u8;
}

fn load_tgx(name: &str) {

    let contents = fs::read(name).unwrap();

    println!("contents.size: {}", contents.len());

    let mut index = 0;

    let mut W: u32 = 0;
    let mut H: u32 = 0;

    W = ((contents[0]) as u32) + ((contents[1] as u32) << 8);
    H = ((contents[4]) as u32) + ((contents[5] as u32) << 8);

    let mut image = Image::new(W, H);
    
    index = 8;

    loop {
        if index >= contents.len() {
            break;
        }

        let opt = (contents[index]) >> 5;
        let count = ((contents[index]) & 0b11111) + 1;

        index += 1;

        if opt == 0b000 {
            for _ in 0..count {
                let color_raw: u32 = ((contents[index + 0]) as u32) + ((contents[index + 1] as u32) << 8);
                index += 2;

                let mut color_b: u8 = 0;
                let mut color_g: u8 = 0;
                let mut color_r: u8 = 0;

                parse_pixel(color_raw as u16, &mut color_r, &mut color_g, &mut color_b);

                image.add_repeat(1, color_r, color_g, color_b);
            }
        }
        if opt == 0b100 {
            image.add_newline();
        }
        if opt == 0b010 {
            let color_raw: u32 = ((contents[index + 0]) as u32) + ((contents[index + 1] as u32) << 8);
            index += 2;

            let mut color_b: u8 = 0;
            let mut color_g: u8 = 0;
            let mut color_r: u8 = 0;

            parse_pixel(color_raw as u16, &mut color_r, &mut color_g, &mut color_b);

            image.add_repeat(count, color_r, color_g, color_b);
        }
        if opt == 0b001 {
            image.add_repeat(count, 0, 0, 0);
        }
    }

    let new_filename = name.to_owned() + ".bmp";
    image.write_to_bmp(new_filename.as_str());

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        println!("FILE: {}", filename);
        
        let start = Instant::now();
        load_tgx(filename);
        let duration = start.elapsed();
        println!("Time elapsed: {:?} ms", duration.as_millis());
    }
    else {
        println!("Please set path to tgx file");
        return
    }
}
