# Videos

The **Videos** section of the settings panel appears when the list contains videos. Every video
in the list is converted with the same settings. All conversions use the FFmpeg bundled with the
app.

## Formats

| Format | Good for | Video codec | Audio codec |
|---|---|---|---|
| MP4 | Plays almost everywhere | H.264 | AAC |
| MOV | QuickTime. Common on Mac. | H.264 | AAC |
| MKV | Flexible container | H.264 | AAC |
| WebM | Open web format. Small files. | VP9 | Opus |
| M4V | Apple's MP4 variant | H.264 | AAC |
| AVI | Legacy Windows format | MPEG-4 Part 2 | MP3 |
| FLV | Legacy Flash video | H.264 | AAC |
| GIF | Silent animation. Keep clips short. | GIF | None |

MP4, MOV and M4V files are written so they can start playing before the whole file has loaded.
H.264 and VP9 video is written as 8-bit 4:2:0 colour, which every player supports, so HDR and
10-bit sources lose their extra range.

## Quality

**High**, **Balanced** (the default) and **Small** trade file size against picture and sound
quality. The quality option is hidden for GIF.

| Codec | High | Balanced | Small |
|---|---|---|---|
| H.264 | CRF 18, audio 192 kbit/s | CRF 23, audio 160 kbit/s | CRF 28, audio 128 kbit/s |
| VP9 (WebM) | CRF 28, audio 160 kbit/s | CRF 33, audio 128 kbit/s | CRF 38, audio 96 kbit/s |
| MPEG-4 (AVI) | q 2, audio 192 kbit/s | q 4, audio 160 kbit/s | q 7, audio 128 kbit/s |

Lower CRF and q values mean higher quality and larger files.

## Resolution

Choose **Original**, **2160p (4K)**, **1440p**, **1080p**, **720p** or **480p**. The number limits
the shorter side, so a 1080×1920 portrait video at **720p** becomes 720×1280. Proportions are
kept and videos are never enlarged.

Width and height are rounded down to even numbers, which most encoders require.

## Frame rate

Choose **Original**, **60 fps**, **30 fps**, **24 fps** or **15 fps**. The choice is a maximum:
faster videos drop frames to match it, and slower ones keep their own frame rate.

## GIF

- Audio is removed.
- The colour palette is built from the clip itself, so colours stay close to the original.
- With frame rate set to **Original**, GIFs use at most 15 fps.
- GIFs are at most 480 pixels on the shorter side, so **Resolution** choices above **480p** make no
  difference. Long clips still make large files.
- The video is read twice, once to build the palette and once to write the GIF. The percentage
  covers both.
