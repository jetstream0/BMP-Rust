use std::fs;
use std::convert::TryInto;
//use std::io::ErrorKind;

/*Documentation - important links
https://docs.microsoft.com/en-us/windows/win32/gdi/bitmap-header-types
https://en.wikipedia.org/wiki/BMP_file_format#File_structure
http://fileformats.archiveteam.org/wiki/BMP
*/

//Errors
pub enum ErrorKind {
  Unsupported,
  DoesNotExist,
  WrongFileType,
  UseExtraBitMasks,
}

impl ErrorKind {
    fn as_str(&self) -> &str {
      match *self {
        ErrorKind::Unsupported => "File is unsupported",
        ErrorKind::DoesNotExist => "Requested object does not exist",
        ErrorKind::WrongFileType => "Wrong file type. Must be a .bmp file",
        ErrorKind::UseExtraBitMasks => "Use extra bit masks instead",
      }
    }
}

//File header
struct BITMAPFILEHEADER {
  bfType: String,
  bfSize: u32,
  bfReserved1: Vec<u8>,
  bfReserved2: Vec<u8>,
  bfOffBits: u16,
}

//DIB Headers
struct BITMAPCOREHEADER {
  size: u16,
  width: u32,
  height: u32,
  planes: u16,
  bitcount: u16,
}

//if biCompression is BI_ALPHABITFIELDS or BI_BITFIELDS 
struct BITMAPINFOHEADER {
  size: u16,
  width: u32,
  //biHeight can be negative
  height: i32,
  planes: u16,
  bitcount: u16,
  compression: String,
  sizeimage: u32,
  XPelsPerMeter: u32,
  YPelsPerMeter: u32,
  ClrUsed: u32,
  ClrImportant: u32,
}

struct BITMAPV4HEADER {
  size: u16,
  width: u32,
  //bV4Height can be negative
  height: i32,
  planes: u16,
  bitcount: u16,
  compression: String,
  sizeimage: u32,
  XPelsPerMeter: u32,
  YPelsPerMeter: u32,
  ClrUsed: u32,
  ClrImportant: u32,
  RedMask: u32,
  GreenMask: u32,
  BlueMask: u32,
  AlphaMask: u32,
  CSType: String,
  //rgb
  Endpoints: [[i32; 3]; 3],
  GammaRed: u32,
  GammaGreen: u32,
  GammaBlue: u32,
}

struct BITMAPV5HEADER {
  size: u16,
  width: u32,
  height: i32,
  planes: u16,
  bitcount: u16,
  compression: String,
  sizeimage: u32,
  XPelsPerMeter: u32,
  YPelsPerMeter: u32,
  ClrUsed: u32,
  ClrImportant: u32,
  RedMask: u32,
  GreenMask: u32,
  BlueMask: u32,
  AlphaMask: u32,
  CSType: String,
  Endpoints: [[i32; 3]; 3],
  GammaRed: u32,
  GammaGreen: u32,
  GammaBlue: u32,
  Intent: String,
  ProfileData: u16,
  ProfileSize: u16,
  Reserved: Vec<u8>,
}

enum DIBHEADER {
  BITMAPCOREHEADER(BITMAPCOREHEADER),
  BITMAPINFOHEADER(BITMAPINFOHEADER),
  BITMAPV4HEADER(BITMAPV4HEADER),
  BITMAPV5HEADER(BITMAPV5HEADER),
}

//rgbtriple and rgbquad
enum ColorTable {
  RGBTRIPLE(Vec<[u8; 3]>),
  RGBQUAD(Vec<[u8; 4]>),
}

//extra bit masks, these are unofficial names
struct BI_BITFIELDS_MASKS {
  red: u32,
  green: u32,
  blue: u32,
}

struct BI_ALPHABITFIELDS_MASKS {
  red: u32,
  green: u32,
  blue: u32,
  alpha: u32,
}

enum EXTRA_BIT_MASKS {
  BI_BITFIELDS_MASKS(BI_BITFIELDS_MASKS),
  BI_ALPHABITFIELDS_MASKS(BI_ALPHABITFIELDS_MASKS),
}

pub struct BMP {
  contents: Vec<u8>,
  from_file: bool,
  //bitmap_file_header: BITMAPFILEHEADER,
  //dib_header: DIBHEADER,
}

impl BMP {
  /*pub fn new() -> BMP {
    return BMP { contents: Vec::new(), from_file: false };
  }*/
  pub fn new_from_file(file_path: &str) -> BMP {
    let contents = fs::read(file_path)
      .expect("Error encountered");
    return BMP { contents: contents, from_file: true, };
  }
  //utilities
  fn bytes_to_int(bytes: [u8; 4]) -> u32 {
    u32::from_le_bytes(bytes)
  }
  fn byte_to_int(byte: u8) -> u8 {
    u8::from_le_bytes([byte])
  }
  fn bytes_to_signed_int(bytes: [u8; 4]) -> i32 {
    i32::from_le_bytes(bytes)
  }
  fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes).to_string()
  }
  fn num_bytes_to_kilobytes(bytes: u32) -> u32 {
    //1024 bytes per kilobyte
    bytes/1024
  }
  //file header related
  fn get_header(&self) -> BITMAPFILEHEADER {
    let header_bytes: &[u8; 14] = self.get_header_bytes();
    return BITMAPFILEHEADER {
      bfType: BMP::bytes_to_string(&header_bytes[..2]),
      bfSize: BMP::bytes_to_int(header_bytes[2..6].try_into().unwrap()),
      bfReserved1: header_bytes[6..8].try_into().unwrap(),
      bfReserved2: header_bytes[8..10].try_into().unwrap(),
      bfOffBits: BMP::bytes_to_int(header_bytes[10..14].try_into().unwrap()) as u16,
    };
  }
  fn get_header_bytes(&self) -> &[u8; 14] {
    //turn slice into array
    self.contents[..14].try_into().unwrap()
  }
  fn get_offset(&self) -> u16 {
    self.get_header().bfOffBits
  }
  pub fn get_size(&self, use_header: bool) -> u32 {
    if use_header {
      return self.get_header().bfSize;
    } else {
      return self.contents.len().try_into().unwrap();
    }
  }
  //dib header related
  fn get_dib_header(&self) -> Result<DIBHEADER, ErrorKind> {
    //this will not work because there may be other data besides the DIB header
    //let dib_size: i32 = self.get_offset()-14;
    //instead we will read the first 4 bytes after the header, which *should* specify the DIB header size, so we can figure out what kind of header it is
    let HEADER_OFFSET = 14;
    let dib_size: u32 = BMP::bytes_to_int(self.contents[HEADER_OFFSET..HEADER_OFFSET+4].try_into().unwrap());
    let dib_header: DIBHEADER;
    match dib_size {
      12 => {
        //"BITMAPCOREHEADER"
        dib_header = DIBHEADER::BITMAPCOREHEADER(BITMAPCOREHEADER {
          size: dib_size as u16,
          width: BMP::bytes_to_int(self.contents[HEADER_OFFSET+4..HEADER_OFFSET+6].try_into().unwrap()),
          height: BMP::bytes_to_int(self.contents[HEADER_OFFSET+6..HEADER_OFFSET+8].try_into().unwrap()),
          planes: BMP::bytes_to_int(self.contents[HEADER_OFFSET+8..HEADER_OFFSET+10].try_into().unwrap()) as u16,
          bitcount: BMP::bytes_to_int(self.contents[HEADER_OFFSET+10..HEADER_OFFSET+12].try_into().unwrap()) as u16,
        });
      },
      40 => {
        //"BITMAPINFOHEADER"
        dib_header = DIBHEADER::BITMAPINFOHEADER(BITMAPINFOHEADER {
          size: dib_size as u16,
          width: BMP::bytes_to_int(self.contents[HEADER_OFFSET+4..HEADER_OFFSET+8].try_into().unwrap()),
          height: BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+8..HEADER_OFFSET+12].try_into().unwrap()),
          planes: BMP::bytes_to_int(self.contents[HEADER_OFFSET+12..HEADER_OFFSET+14].try_into().unwrap()) as u16,
          bitcount: BMP::bytes_to_int(self.contents[HEADER_OFFSET+14..HEADER_OFFSET+16].try_into().unwrap()) as u16,
          compression: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+16..HEADER_OFFSET+20]),
          sizeimage: BMP::bytes_to_int(self.contents[HEADER_OFFSET+20..HEADER_OFFSET+24].try_into().unwrap()),
          XPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+24..HEADER_OFFSET+28].try_into().unwrap()),
          YPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+28..HEADER_OFFSET+32].try_into().unwrap()),
          ClrUsed: BMP::bytes_to_int(self.contents[HEADER_OFFSET+32..HEADER_OFFSET+36].try_into().unwrap()),
          ClrImportant: BMP::bytes_to_int(self.contents[HEADER_OFFSET+36..HEADER_OFFSET+40].try_into().unwrap()),
        });
      },
      108 => {
        //"BITMAPV4HEADER"
        dib_header = DIBHEADER::BITMAPV4HEADER(BITMAPV4HEADER {
          size: dib_size as u16,
          width: BMP::bytes_to_int(self.contents[HEADER_OFFSET+4..HEADER_OFFSET+8].try_into().unwrap()),
          height: BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+8..HEADER_OFFSET+12].try_into().unwrap()),
          planes: BMP::bytes_to_int(self.contents[HEADER_OFFSET+12..HEADER_OFFSET+14].try_into().unwrap()) as u16,
          bitcount: BMP::bytes_to_int(self.contents[HEADER_OFFSET+14..HEADER_OFFSET+16].try_into().unwrap()) as u16,
          compression: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+16..HEADER_OFFSET+20]),
          sizeimage: BMP::bytes_to_int(self.contents[HEADER_OFFSET+20..HEADER_OFFSET+24].try_into().unwrap()),
          XPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+24..HEADER_OFFSET+28].try_into().unwrap()),
          YPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+28..HEADER_OFFSET+32].try_into().unwrap()),
          ClrUsed: BMP::bytes_to_int(self.contents[HEADER_OFFSET+32..HEADER_OFFSET+36].try_into().unwrap()),
          ClrImportant: BMP::bytes_to_int(self.contents[HEADER_OFFSET+36..HEADER_OFFSET+40].try_into().unwrap()),
          RedMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+40..HEADER_OFFSET+44].try_into().unwrap()),
          GreenMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+44..HEADER_OFFSET+48].try_into().unwrap()),
          BlueMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+48..HEADER_OFFSET+52].try_into().unwrap()),
          AlphaMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+52..HEADER_OFFSET+56].try_into().unwrap()),
          CSType: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+56..HEADER_OFFSET+60]),
          //rgb
          Endpoints: [[BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+60..HEADER_OFFSET+64].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+64..HEADER_OFFSET+68].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+68..HEADER_OFFSET+72].try_into().unwrap())], [BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+72..HEADER_OFFSET+76].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+76..HEADER_OFFSET+80].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+80..HEADER_OFFSET+84].try_into().unwrap())], [BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+84..HEADER_OFFSET+88].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+88..HEADER_OFFSET+92].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+92..HEADER_OFFSET+96].try_into().unwrap())]],
          GammaRed: BMP::bytes_to_int(self.contents[HEADER_OFFSET+96..HEADER_OFFSET+100].try_into().unwrap()),
          GammaGreen: BMP::bytes_to_int(self.contents[HEADER_OFFSET+100..HEADER_OFFSET+104].try_into().unwrap()),
          GammaBlue: BMP::bytes_to_int(self.contents[HEADER_OFFSET+104..HEADER_OFFSET+108].try_into().unwrap()),
        });
      },
      124 => {
        //"BITMAPV5HEADER"
        //dword 4 bytes
          //long 4 bytes
          //CIEXYZTRIPLE 36 bytes
        dib_header = DIBHEADER::BITMAPV5HEADER(BITMAPV5HEADER {
          size: dib_size as u16,
          width: BMP::bytes_to_int(self.contents[HEADER_OFFSET+4..HEADER_OFFSET+8].try_into().unwrap()),
          height: BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+8..HEADER_OFFSET+12].try_into().unwrap()),
          planes: BMP::bytes_to_int(self.contents[HEADER_OFFSET+12..HEADER_OFFSET+14].try_into().unwrap()) as u16,
          bitcount: BMP::bytes_to_int(self.contents[HEADER_OFFSET+14..HEADER_OFFSET+16].try_into().unwrap()) as u16,
          compression: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+16..HEADER_OFFSET+20]),
          sizeimage: BMP::bytes_to_int(self.contents[HEADER_OFFSET+20..HEADER_OFFSET+24].try_into().unwrap()),
          XPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+24..HEADER_OFFSET+28].try_into().unwrap()),
          YPelsPerMeter: BMP::bytes_to_int(self.contents[HEADER_OFFSET+28..HEADER_OFFSET+32].try_into().unwrap()),
          ClrUsed: BMP::bytes_to_int(self.contents[HEADER_OFFSET+32..HEADER_OFFSET+36].try_into().unwrap()),
          ClrImportant: BMP::bytes_to_int(self.contents[HEADER_OFFSET+36..HEADER_OFFSET+40].try_into().unwrap()),
          RedMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+40..HEADER_OFFSET+44].try_into().unwrap()),
          GreenMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+44..HEADER_OFFSET+48].try_into().unwrap()),
          BlueMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+48..HEADER_OFFSET+52].try_into().unwrap()),
          AlphaMask: BMP::bytes_to_int(self.contents[HEADER_OFFSET+52..HEADER_OFFSET+56].try_into().unwrap()),
          CSType: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+56..HEADER_OFFSET+60]),
          //rgb
          Endpoints: [[BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+60..HEADER_OFFSET+64].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+64..HEADER_OFFSET+68].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+68..HEADER_OFFSET+72].try_into().unwrap())], [BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+72..HEADER_OFFSET+76].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+76..HEADER_OFFSET+80].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+80..HEADER_OFFSET+84].try_into().unwrap())], [BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+84..HEADER_OFFSET+88].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+88..HEADER_OFFSET+92].try_into().unwrap()), BMP::bytes_to_signed_int(self.contents[HEADER_OFFSET+92..HEADER_OFFSET+96].try_into().unwrap())]],
          GammaRed: BMP::bytes_to_int(self.contents[HEADER_OFFSET+96..HEADER_OFFSET+100].try_into().unwrap()),
          GammaGreen: BMP::bytes_to_int(self.contents[HEADER_OFFSET+100..HEADER_OFFSET+104].try_into().unwrap()),
          GammaBlue: BMP::bytes_to_int(self.contents[HEADER_OFFSET+104..HEADER_OFFSET+108].try_into().unwrap()),
          Intent: BMP::bytes_to_string(&self.contents[HEADER_OFFSET+108..HEADER_OFFSET+112]),
          ProfileData: BMP::bytes_to_int(self.contents[HEADER_OFFSET+112..HEADER_OFFSET+116].try_into().unwrap()) as u16,
          ProfileSize: BMP::bytes_to_int(self.contents[HEADER_OFFSET+116..HEADER_OFFSET+120].try_into().unwrap()) as u16,
          Reserved: self.contents[HEADER_OFFSET+120..HEADER_OFFSET+124].try_into().unwrap(),
        });
      },
      _ => {
        //"unsupported"
        return Err(ErrorKind::Unsupported);
      },
    }
    return Ok(dib_header);
  }
  //extra bit masks
  fn get_extra_bit_masks(&self) -> Result<EXTRA_BIT_MASKS, ErrorKind> {
    //should be mutable instead of redefined, maybe
    let dib_header = self.get_dib_header();
    let dib_header = match dib_header {
      Ok(returned_dib_header) => returned_dib_header,
      Err(e) => return Err(e),
    };
    match dib_header {
      DIBHEADER::BITMAPINFOHEADER(BITMAPINFOHEADER) => {
        //see previous comment, should be mutable instead of redefined, maybe
        let dib_header = BITMAPINFOHEADER;
        //offset should be 14+40
        let TOTAL_OFFSET = 54;
        if dib_header.compression == "BI_BITFIELDS" {
          return Ok(EXTRA_BIT_MASKS::BI_BITFIELDS_MASKS(BI_BITFIELDS_MASKS {
            red: BMP::bytes_to_int(self.contents[TOTAL_OFFSET..TOTAL_OFFSET+4].try_into().unwrap()),
            green: BMP::bytes_to_int(self.contents[TOTAL_OFFSET+4..TOTAL_OFFSET+8].try_into().unwrap()),
            blue: BMP::bytes_to_int(self.contents[TOTAL_OFFSET+8..TOTAL_OFFSET+12].try_into().unwrap()),
          }));
        } else if dib_header.compression == "BI_ALPHABITFIELDS" {
          return Ok(EXTRA_BIT_MASKS::BI_ALPHABITFIELDS_MASKS(BI_ALPHABITFIELDS_MASKS {
            red: BMP::bytes_to_int(self.contents[TOTAL_OFFSET..TOTAL_OFFSET+4].try_into().unwrap()),
            green: BMP::bytes_to_int(self.contents[TOTAL_OFFSET+4..TOTAL_OFFSET+8].try_into().unwrap()),
            blue: BMP::bytes_to_int(self.contents[TOTAL_OFFSET+8..TOTAL_OFFSET+12].try_into().unwrap()),
            alpha: BMP::bytes_to_int(self.contents[TOTAL_OFFSET+12..TOTAL_OFFSET+16].try_into().unwrap()),
          }));
        } else {
          return Err(ErrorKind::DoesNotExist);
        }
      },
      _ => return Err(ErrorKind::DoesNotExist),
    }
  }
  //color table
  //in between pixel array and everything else, I guess?
  //update: use the dib header's 'size' attribute - the actual size
  //return some kind of vector/array
  fn get_color_table(&self) -> Result<ColorTable, ErrorKind> {
    let dib_header = self.get_dib_header();
    let dib_header = match dib_header {
      Ok(returned_dib_header) => returned_dib_header,
      Err(e) => return Err(e),
    };
    //match (?) and extract header, get size
    //14 is the file header size
    let mut offset: u16 = 14;
    //where the actual pixel data starts, so the color table must end sometime before
    let end: u16;
    //either rgbtriple or masks or 
    let data_type: &str;
    match dib_header {
      /*DIBHEADER::BITMAPCOREHEADER(b) | DIBHEADER::BITMAPINFOHEADER(b) | DIBHEADER::BITMAPV4HEADER(b) | DIBHEADER::BITMAPV5HEADER(b) => {
        size = b.size;
      }*/
      DIBHEADER::BITMAPCOREHEADER(BITMAPCOREHEADER) => {
        //https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapcoreinfo
        offset += BITMAPCOREHEADER.size;
        end = self.get_header().bfOffBits;
        //RGBTRIPLE, 3 bytes
        data_type = "rgbtriple";
      },
      DIBHEADER::BITMAPINFOHEADER(bih) => {
        //16 bit array instead of rgbquad is possible, but should not be used if file is "stored in a file or transferred to another application" https://www.digicamsoft.com/bmp/bmp.html
        offset += bih.size;
        end = self.get_header().bfOffBits;
        //https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfo
        //if compression is BI_RGB, using RGBQUAD 
        //size of array is biClrUsed
        if bih.compression == "BI_BITFIELDS" && (bih.bitcount == 16 || bih.bitcount == 32) {
          //extra bit masks, not color table. return error, or maybe extra bit masks? hmm
          return Err(ErrorKind::UseExtraBitMasks);
        } else if bih.compression == "BI_RGB" && bih.bitcount >= 16 {
          //no color table
          //color table used for optimizing color palette or something instead, idk
          return Err(ErrorKind::DoesNotExist);
        } else {
          data_type = "rgbquad";
        }
      },
      /*DIBHEADER::BITMAPV4HEADER(bih) => {
        //
      },
      DIBHEADER::BITMAPV5HEADER(bih) => {
        //
      },*/
      _ => {
        return Err(ErrorKind::DoesNotExist);
      },
    };
    let color_table: ColorTable;
    if "rgbtriple" == data_type {
      let mut color_table_vec: Vec::<[u8; 3]> = Vec::new();
      //3 bytes
      for i in 0..(f64::from((end-offset)/3).floor() as i64) {
        let ii = i as u16;
        color_table_vec.push([BMP::byte_to_int(self.contents[(offset+ii*3) as usize]) as u8, BMP::byte_to_int(self.contents[(offset+ii*3+1) as usize]) as u8, BMP::byte_to_int(self.contents[(offset+ii*3+2) as usize]) as u8]);
      }
      color_table = ColorTable::RGBTRIPLE(color_table_vec);
    } else /*if "rgbquad" == data_type*/ {
      let mut color_table_vec: Vec::<[u8; 4]> = Vec::new();
      //4 bytes
      for i in 0..(f64::from((end-offset)/4).floor() as i64) {
        let ii = i as u16;
        color_table_vec.push([BMP::byte_to_int(self.contents[(offset+ii*4) as usize]) as u8, BMP::byte_to_int(self.contents[(offset+ii*4+1) as usize]) as u8, BMP::byte_to_int(self.contents[(offset+ii*4+2) as usize]) as u8, BMP::byte_to_int(self.contents[(offset+ii*4+3) as usize]) as u8]);
      }
      color_table = ColorTable::RGBQUAD(color_table_vec);
    }
    return Ok(color_table);
  }
  //pixel array
  //location here is told
  //ICC color profile
}