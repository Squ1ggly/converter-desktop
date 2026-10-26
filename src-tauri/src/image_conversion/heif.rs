// Reads the parts of a HEIF container (HEIC, HEIF, AVIF) that FFmpeg's
// default decode leaves out: which item is the main image's transparency, and
// how the main image is rotated and mirrored.
//
// A HEIF file stores images as items in its `meta` box. `pitm` names the main
// (primary) item, `iinf` gives each item's type, `iref` links items (an alpha
// plane points at its image with `auxl`), `ipco` lists properties and `ipma`
// assigns them to items. Every read is bounds-checked: a damaged file gives
// None, never a panic.

use image::DynamicImage;

// The two names HEIF uses for an alpha plane: the AVIF one and the HEVC one.
const ALPHA_AUXILIARY_TYPES: [&[u8]; 2] = [
    b"urn:mpeg:mpegB:cicp:systems:auxiliary:alpha",
    b"urn:mpeg:hevc:2015:auxid:1",
];
const FULL_BOX_HEADER_LENGTH: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeifItem {
    pub id: u32,
    // A grid item is drawn from tiles, which FFmpeg exposes as a stream group.
    pub is_grid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transform {
    // Anticlockwise, in quarter turns (`irot`).
    Rotate { quarter_turns: u8 },
    // `imir`: 0 swaps top and bottom, 1 swaps left and right.
    Mirror { axis: u8 },
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeifImage {
    pub primary: HeifItem,
    pub alpha: Option<HeifItem>,
    // In the order they apply.
    pub primary_transforms: Vec<Transform>,
}

pub fn read_heif_image(file: &[u8]) -> Option<HeifImage> {
    let meta_payload = find_box(file, *b"meta")?;
    let meta = MetaBoxes::read(meta_payload.get(FULL_BOX_HEADER_LENGTH..)?);

    let primary_id = read_primary_item_id(meta.pitm?)?;
    let item_types = meta.iinf.map(read_item_types).unwrap_or_default();
    let properties = read_properties(meta.ipco?);
    let associations = read_associations(meta.ipma?)?;
    let alpha_references = meta.iref.map(read_alpha_references).unwrap_or_default();

    let alpha_id = alpha_references
        .iter()
        .filter(|reference| reference.image_id == primary_id)
        .map(|reference| reference.alpha_id)
        .find(|candidate_id| {
            item_properties(*candidate_id, &associations, &properties)
                .any(Property::is_alpha_auxiliary)
        });
    let primary_transforms = item_properties(primary_id, &associations, &properties)
        .filter_map(Property::as_transform)
        .collect();

    let heif_image = HeifImage {
        primary: item_with_type(primary_id, &item_types),
        alpha: alpha_id.map(|id| item_with_type(id, &item_types)),
        primary_transforms,
    };
    Some(heif_image)
}

// Applies the primary item's rotation and mirroring, as FFmpeg would have.
pub fn apply_transforms(image: DynamicImage, transforms: &[Transform]) -> DynamicImage {
    let mut transformed_image = image;
    for transform in transforms {
        transformed_image = match transform {
            Transform::Rotate { quarter_turns: 1 } => transformed_image.rotate270(),
            Transform::Rotate { quarter_turns: 2 } => transformed_image.rotate180(),
            Transform::Rotate { quarter_turns: 3 } => transformed_image.rotate90(),
            Transform::Mirror { axis: 0 } => transformed_image.flipv(),
            Transform::Mirror { axis: _ } => transformed_image.fliph(),
            Transform::Rotate { quarter_turns: _ } => transformed_image,
        };
    }
    transformed_image
}

fn item_with_type(id: u32, item_types: &[(u32, [u8; 4])]) -> HeifItem {
    let is_grid = item_types
        .iter()
        .any(|(item_id, item_type)| *item_id == id && item_type == b"grid");
    HeifItem { id, is_grid }
}

#[derive(Default)]
struct MetaBoxes<'a> {
    pitm: Option<&'a [u8]>,
    iinf: Option<&'a [u8]>,
    iref: Option<&'a [u8]>,
    ipco: Option<&'a [u8]>,
    ipma: Option<&'a [u8]>,
}

impl<'a> MetaBoxes<'a> {
    fn read(meta_children: &'a [u8]) -> MetaBoxes<'a> {
        let mut meta_boxes = MetaBoxes::default();
        for (box_type, payload) in BoxIterator::new(meta_children) {
            match &box_type {
                b"pitm" => meta_boxes.pitm = Some(payload),
                b"iinf" => meta_boxes.iinf = Some(payload),
                b"iref" => meta_boxes.iref = Some(payload),
                b"iprp" => {
                    meta_boxes.ipco = find_box(payload, *b"ipco");
                    meta_boxes.ipma = find_box(payload, *b"ipma");
                }
                _ => {}
            }
        }
        meta_boxes
    }
}

enum Property<'a> {
    AuxiliaryType(&'a [u8]),
    Transform(Transform),
    Other,
}

impl Property<'_> {
    fn is_alpha_auxiliary(&self) -> bool {
        let Property::AuxiliaryType(auxiliary_type) = self else {
            return false;
        };
        ALPHA_AUXILIARY_TYPES.contains(auxiliary_type)
    }

    fn as_transform(&self) -> Option<Transform> {
        let Property::Transform(transform) = self else {
            return None;
        };
        Some(*transform)
    }
}

fn read_properties(ipco_payload: &[u8]) -> Vec<Property<'_>> {
    let properties = BoxIterator::new(ipco_payload)
        .map(|(box_type, payload)| read_property(box_type, payload))
        .collect();
    properties
}

fn read_property(box_type: [u8; 4], payload: &[u8]) -> Property<'_> {
    let first_byte = payload.first().copied();
    let property = match (&box_type, first_byte) {
        (b"irot", Some(angle)) => Property::Transform(Transform::Rotate {
            quarter_turns: angle & 0b11,
        }),
        (b"imir", Some(axis)) => Property::Transform(Transform::Mirror { axis: axis & 1 }),
        (b"auxC", _) => {
            read_auxiliary_type(payload).map_or(Property::Other, Property::AuxiliaryType)
        }
        _ => Property::Other,
    };
    property
}

// `auxC` is a full box holding a NUL-terminated type name.
fn read_auxiliary_type(payload: &[u8]) -> Option<&[u8]> {
    let name_and_rest = payload.get(FULL_BOX_HEADER_LENGTH..)?;
    let name_length = name_and_rest
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(name_and_rest.len());
    name_and_rest.get(..name_length)
}

// (item id, 1-based property indices), from `ipma`.
type Associations = Vec<(u32, Vec<usize>)>;

fn read_associations(ipma_payload: &[u8]) -> Option<Associations> {
    let mut reader = ByteReader::new(ipma_payload);
    let version = reader.read_u8()?;
    let flags = reader.read_number(3)?;
    let has_wide_indices = flags & 1 == 1;
    let item_id_length = if version < 1 { 2 } else { 4 };
    let entry_count = reader.read_number(4)?;

    let mut associations = Vec::new();
    for _ in 0..entry_count {
        let item_id = reader.read_number(item_id_length)?;
        let association_count = reader.read_u8()?;
        let mut property_indices = Vec::new();
        for _ in 0..association_count {
            property_indices.push(read_property_index(&mut reader, has_wide_indices)?);
        }
        associations.push((u32::try_from(item_id).ok()?, property_indices));
    }
    Some(associations)
}

// The top bit marks an essential property; the rest is the index.
fn read_property_index(reader: &mut ByteReader, has_wide_indices: bool) -> Option<usize> {
    let property_index = if has_wide_indices {
        reader.read_number(2)? & 0x7fff
    } else {
        reader.read_number(1)? & 0x7f
    };
    Some(property_index)
}

fn item_properties<'a>(
    item_id: u32,
    associations: &'a Associations,
    properties: &'a [Property<'a>],
) -> impl Iterator<Item = &'a Property<'a>> {
    associations
        .iter()
        .filter(move |(associated_id, _)| *associated_id == item_id)
        .flat_map(|(_, property_indices)| property_indices)
        .filter_map(|property_index| properties.get(property_index.checked_sub(1)?))
}

fn read_primary_item_id(pitm_payload: &[u8]) -> Option<u32> {
    let mut reader = ByteReader::new(pitm_payload);
    let version = reader.read_u8()?;
    reader.skip(3)?;
    let id_length = if version == 0 { 2 } else { 4 };
    let primary_id = u32::try_from(reader.read_number(id_length)?).ok()?;
    Some(primary_id)
}

// (item id, item type) from each `infe` in `iinf`.
fn read_item_types(iinf_payload: &[u8]) -> Vec<(u32, [u8; 4])> {
    let entry_count_length = if iinf_payload.first() == Some(&0) {
        2
    } else {
        4
    };
    let Some(entries) = iinf_payload.get(FULL_BOX_HEADER_LENGTH + entry_count_length..) else {
        return Vec::new();
    };
    let item_types = BoxIterator::new(entries)
        .filter(|(box_type, _)| box_type == b"infe")
        .filter_map(|(_, payload)| read_item_type(payload))
        .collect();
    item_types
}

// Versions 2 and 3 of `infe`, the ones that carry an item type.
fn read_item_type(infe_payload: &[u8]) -> Option<(u32, [u8; 4])> {
    let mut reader = ByteReader::new(infe_payload);
    let version = reader.read_u8()?;
    reader.skip(3)?;
    let id_length = match version {
        2 => 2,
        3 => 4,
        _ => return None,
    };
    let item_id = u32::try_from(reader.read_number(id_length)?).ok()?;
    reader.skip(2)?;
    let item_type = reader.read_box_type()?;
    Some((item_id, item_type))
}

struct AlphaReference {
    alpha_id: u32,
    image_id: u32,
}

// `auxl` references, from an auxiliary item to the image it belongs to.
fn read_alpha_references(iref_payload: &[u8]) -> Vec<AlphaReference> {
    let id_length = if iref_payload.first() == Some(&0) {
        2
    } else {
        4
    };
    let Some(references) = iref_payload.get(FULL_BOX_HEADER_LENGTH..) else {
        return Vec::new();
    };
    let alpha_references = BoxIterator::new(references)
        .filter(|(box_type, _)| box_type == b"auxl")
        .filter_map(|(_, payload)| read_auxiliary_reference(payload, id_length))
        .flatten()
        .collect();
    alpha_references
}

fn read_auxiliary_reference(payload: &[u8], id_length: usize) -> Option<Vec<AlphaReference>> {
    let mut reader = ByteReader::new(payload);
    let alpha_id = u32::try_from(reader.read_number(id_length)?).ok()?;
    let reference_count = reader.read_number(2)?;
    let mut alpha_references = Vec::new();
    for _ in 0..reference_count {
        let image_id = u32::try_from(reader.read_number(id_length)?).ok()?;
        alpha_references.push(AlphaReference { alpha_id, image_id });
    }
    Some(alpha_references)
}

fn find_box(bytes: &[u8], wanted_type: [u8; 4]) -> Option<&[u8]> {
    BoxIterator::new(bytes)
        .find(|(box_type, _)| *box_type == wanted_type)
        .map(|(_, payload)| payload)
}

// Yields (type, payload) for each box in `bytes`, stopping at the first one
// that doesn't fit.
struct BoxIterator<'a> {
    reader: ByteReader<'a>,
}

impl<'a> BoxIterator<'a> {
    fn new(bytes: &'a [u8]) -> BoxIterator<'a> {
        BoxIterator {
            reader: ByteReader::new(bytes),
        }
    }
}

impl<'a> Iterator for BoxIterator<'a> {
    type Item = ([u8; 4], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let box_start = self.reader.position;
        let declared_size = self.reader.read_number(4)?;
        let box_type = self.reader.read_box_type()?;
        // 1 means a 64-bit size follows; 0 means the box runs to the end.
        let box_size = match declared_size {
            0 => self.reader.bytes.len() - box_start,
            1 => self.reader.read_number(8)?,
            size => size,
        };
        let box_end = box_start.checked_add(box_size)?;
        let payload = self.reader.bytes.get(self.reader.position..box_end)?;
        self.reader.position = box_end;
        Some((box_type, payload))
    }
}

struct ByteReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &'a [u8]) -> ByteReader<'a> {
        ByteReader { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.position.checked_add(length)?;
        let taken = self.bytes.get(self.position..end)?;
        self.position = end;
        Some(taken)
    }

    fn skip(&mut self, length: usize) -> Option<()> {
        self.take(length)?;
        Some(())
    }

    fn read_u8(&mut self) -> Option<u8> {
        let byte = *self.take(1)?.first()?;
        Some(byte)
    }

    // Big-endian, up to 8 bytes.
    fn read_number(&mut self, length: usize) -> Option<usize> {
        let number = self.take(length)?.iter().try_fold(0_usize, |total, byte| {
            total.checked_mul(256)?.checked_add(usize::from(*byte))
        })?;
        Some(number)
    }

    fn read_box_type(&mut self) -> Option<[u8; 4]> {
        let box_type = self.take(4)?.try_into().ok()?;
        Some(box_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(box_type: [u8; 4], payload: &[u8]) -> Vec<u8> {
        let box_length = u32::try_from(8 + payload.len()).unwrap();
        [&box_length.to_be_bytes()[..], &box_type, payload].concat()
    }

    fn full_box(box_type: [u8; 4], version: u8, body: &[u8]) -> Vec<u8> {
        make_box(box_type, &[&[version, 0, 0, 0][..], body].concat())
    }

    fn item_info(item_id: u16, item_type: [u8; 4]) -> Vec<u8> {
        full_box(
            *b"infe",
            2,
            &[&item_id.to_be_bytes()[..], &[0, 0], &item_type, b"\0"].concat(),
        )
    }

    // A grid image (item 1) with an alpha plane (item 2), turned a quarter
    // anticlockwise and then mirrored left to right. Item 3 is a tile.
    fn sample_file() -> Vec<u8> {
        let alpha_type = b"urn:mpeg:mpegB:cicp:systems:auxiliary:alpha\0";
        let properties = [
            full_box(*b"auxC", 0, alpha_type),
            make_box(*b"irot", &[1]),
            make_box(*b"imir", &[1]),
        ]
        .concat();
        // Item 1: irot then imir. Item 2: auxC (essential).
        let associations = [
            &2_u32.to_be_bytes()[..],
            &[0, 1, 2, 0x82, 0x83],
            &[0, 2, 1, 0x81],
        ]
        .concat();
        let item_infos = [
            item_info(1, *b"grid"),
            item_info(2, *b"av01"),
            item_info(3, *b"av01"),
        ];
        let meta_children = [
            full_box(*b"pitm", 0, &[0, 1]),
            full_box(*b"iinf", 0, &[&[0, 3][..], &item_infos.concat()].concat()),
            full_box(*b"iref", 0, &make_box(*b"auxl", &[0, 2, 0, 1, 0, 1])),
            make_box(
                *b"iprp",
                &[
                    make_box(*b"ipco", &properties),
                    full_box(*b"ipma", 0, &associations),
                ]
                .concat(),
            ),
        ]
        .concat();
        [
            make_box(*b"ftyp", b"avifmif1"),
            full_box(*b"meta", 0, &meta_children),
            make_box(*b"mdat", &[0; 16]),
        ]
        .concat()
    }

    #[test]
    fn finds_the_alpha_plane_and_the_transforms_of_the_main_image() {
        let heif_image = read_heif_image(&sample_file()).expect("reads the container");

        assert_eq!(
            heif_image,
            HeifImage {
                primary: HeifItem {
                    id: 1,
                    is_grid: true
                },
                alpha: Some(HeifItem {
                    id: 2,
                    is_grid: false
                }),
                primary_transforms: vec![
                    Transform::Rotate { quarter_turns: 1 },
                    Transform::Mirror { axis: 1 },
                ],
            }
        );
    }

    #[test]
    fn an_auxiliary_image_that_is_not_alpha_is_ignored() {
        let file_with_depth = replace_bytes(&sample_file(), b"auxiliary:alpha", b"auxiliary:depth");

        let heif_image = read_heif_image(&file_with_depth).expect("reads the container");

        assert_eq!(heif_image.alpha, None);
    }

    fn replace_bytes(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let position = bytes
            .windows(from.len())
            .position(|window| window == from)
            .unwrap();
        [&bytes[..position], to, &bytes[position + from.len()..]].concat()
    }

    #[test]
    fn damaged_files_are_read_without_panicking() {
        let file = sample_file();
        for length in 0..file.len() {
            let _ = read_heif_image(&file[..length]);
        }
        let mut corrupted_file = file.clone();
        for index in 0..corrupted_file.len() {
            corrupted_file[index] = 0xff;
            let _ = read_heif_image(&corrupted_file);
            corrupted_file[index] = file[index];
        }
        assert_eq!(read_heif_image(b"not a heif file"), None);
    }

    #[test]
    fn transforms_turn_and_mirror_the_image() {
        let mut image = image::RgbImage::new(4, 2);
        image.put_pixel(0, 0, image::Rgb([255, 0, 0]));

        let turned = apply_transforms(
            DynamicImage::ImageRgb8(image),
            &[
                Transform::Rotate { quarter_turns: 1 },
                Transform::Mirror { axis: 1 },
            ],
        )
        .to_rgb8();

        assert_eq!(turned.dimensions(), (2, 4));
        assert_eq!(turned.get_pixel(1, 3).0, [255, 0, 0]);
    }
}
