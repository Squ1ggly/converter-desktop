# Images

The **Images** section of the settings panel appears when the list contains images. Every image
in the list is converted with the same settings.

## Formats

Any supported image can be converted to any of these formats. The picker shows a short
description under the selected format.

| Format | Good for | Quality slider |
|---|---|---|
| JPG | Photos. Opens anywhere. No transparency. | Yes |
| PNG | Lossless copies, screenshots and graphics. Keeps transparency. | No |
| WebP | Smaller files than JPG. Keeps transparency. | Yes |
| AVIF | The smallest files. Modern browsers only. | Yes |
| HEIC | Apple's photo format. macOS only. | Yes |
| GIF | Simple graphics. 256 colours. | No |
| BMP | Uncompressed. Large files. | No |
| TIFF | Lossless. Print and archiving. | No |
| PDF | One page per image. | Yes |
| SVG | Logos and icons, traced to vector shapes. | No |

**HEIC output is only offered on macOS**, because it uses the HEIC encoder built into macOS. On
Windows and Linux, HEIC files can still be opened and converted to other formats.

## Quality

Formats marked "Yes" above show a **Quality** slider from 1 to 100, running from "Smaller" to
"Sharper". The default is 85. Other formats ignore it.

## Size

| Option | What it does |
|---|---|
| Original | Keeps the original pixel size |
| Scale | Scales both sides by a percentage, from 1% to 1000%. Starts at 50%. |
| Fit | Scales the image to fit inside a width × height box in pixels, keeping its proportions. Leave a side blank to scale by the other. |

**Fit** enlarges images smaller than the box as well as shrinking larger ones. With both sides
blank, the size is unchanged.

SVG and PDF sources are drawn at the requested size, so they stay sharp when enlarged.

## Transparency

PNG, WebP, AVIF and TIFF keep transparent areas. JPG, BMP and PDF can't store transparency, so
transparent areas become white, as they would in an image editor.

## Rotation

When a JPG, PNG, WebP or TIFF has an EXIF orientation tag, as most phone photos do, the image is
rotated to match. The converted file is upright without relying on the tag.

## Notes by format

- **PDF input**: only the first page is converted. Pages are drawn at 150 DPI for "Original" size,
  on a white background.
- **PDF output**: each image becomes a one-page PDF with the page the same size as the image
  (one pixel per point). The image inside is stored as a JPG at the chosen quality.
- **SVG input**: drawn with the fonts installed on your computer, so text may look different from
  the machine that made the file.
- **SVG output**: the image is traced into vector shapes. This suits logos and flat graphics more
  than photos. Images larger than 2000 pixels on either side are shrunk to fit 2000 × 2000 before
  tracing.
- **GIF and WebP input**: for animated files, only the first frame is converted.
- **HEIC, HEIF and AVIF input** are read with the bundled FFmpeg.
- **Misnamed files**: if a file can't be read as the format its extension says (for example an
  iPhone HEIC renamed to `.jpg`), the app tries FFmpeg before reporting an error.
