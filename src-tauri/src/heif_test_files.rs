// Adds HEIF transform properties (`irot`, `imir`) to AVIF and HEIC files, so
// orientation can be tested on every platform. FFmpeg can't write them.
//
// The file is a list of boxes. Properties live in meta/iprp/ipco and are tied
// to items in meta/iprp/ipma. Item data is found through file offsets in
// meta/iloc, which move when the meta box grows.

const BOX_HEADER_LENGTH: usize = 8;

#[derive(Clone, Copy, Debug)]
pub enum Transform {
    // Anticlockwise, in steps of 90 degrees.
    Rotate { quarter_turns: u8 },
    // 0 swaps top and bottom, 1 swaps left and right (ISO/IEC 23008-12:2022).
    Mirror { axis: u8 },
}

impl Transform {
    fn property_box(self) -> Vec<u8> {
        let (box_type, value) = match self {
            Transform::Rotate { quarter_turns } => (b"irot", quarter_turns),
            Transform::Mirror { axis } => (b"imir", axis),
        };
        let property_box = make_box(*box_type, &[value]);
        property_box
    }
}

struct RawBox {
    box_type: [u8; 4],
    payload: Vec<u8>,
}

// Adds each transform, in order, to the primary item (item 1).
pub fn with_transforms(file: &[u8], transforms: &[Transform]) -> Vec<u8> {
    let (meta_start_offset, meta_end_offset) = find_meta_box(file);
    let mut meta = parse_boxes(&file[meta_start_offset..meta_end_offset]).remove(0);

    let property_boxes: Vec<Vec<u8>> = transforms
        .iter()
        .map(|transform| transform.property_box())
        .collect();
    let growth = property_boxes.iter().map(Vec::len).sum::<usize>() + transforms.len();

    let (full_box_header, children) = meta.payload.split_at(4);
    let mut meta_children = parse_boxes(children);
    for child in &mut meta_children {
        match &child.box_type {
            b"iprp" => child.payload = add_properties(&child.payload, &property_boxes),
            b"iloc" => shift_item_offsets(&mut child.payload, meta_end_offset, growth),
            _ => {}
        }
    }
    meta.payload = [full_box_header.to_vec(), join_boxes(&meta_children)].concat();

    let patched_file = [
        &file[..meta_start_offset],
        &join_boxes(&[meta]),
        &file[meta_end_offset..],
    ]
    .concat();
    patched_file
}

// Top-level boxes before meta (just ftyp) never use 64-bit sizes.
fn find_meta_box(file: &[u8]) -> (usize, usize) {
    let mut position = 0;
    loop {
        let box_size = read_number(file, position, 4);
        if &file[position + 4..position + 8] == b"meta" {
            return (position, position + box_size);
        }
        position += box_size;
    }
}

fn add_properties(iprp_payload: &[u8], property_boxes: &[Vec<u8>]) -> Vec<u8> {
    let mut children = parse_boxes(iprp_payload);
    let existing_property_count = children
        .iter()
        .find(|child| &child.box_type == b"ipco")
        .map(|ipco| parse_boxes(&ipco.payload).len())
        .expect("an ipco box");

    for child in &mut children {
        if &child.box_type == b"ipco" {
            child.payload.extend(property_boxes.concat());
        }
        if &child.box_type == b"ipma" {
            add_associations(
                &mut child.payload,
                existing_property_count,
                property_boxes.len(),
            );
        }
    }
    let iprp_with_properties = join_boxes(&children);
    iprp_with_properties
}

// ipma lists, for each item, which properties apply, by 1-based index.
fn add_associations(
    ipma_payload: &mut Vec<u8>,
    existing_property_count: usize,
    added_count: usize,
) {
    let version = ipma_payload[0];
    let has_wide_indices = ipma_payload[3] & 1 == 1;
    assert!(
        !has_wide_indices,
        "2-byte property indices aren't needed here"
    );
    let item_id_length = if version < 1 { 2 } else { 4 };

    // The first entry is item 1, the primary item, in files from FFmpeg and sips.
    let first_entry_start = 4 + 4;
    let association_count_position = first_entry_start + item_id_length;
    let association_count = usize::from(ipma_payload[association_count_position]);
    let insert_position = association_count_position + 1 + association_count;

    let new_associations: Vec<u8> = (1..=added_count)
        .map(|offset| {
            let property_index = u8::try_from(existing_property_count + offset).unwrap();
            0x80 | property_index
        })
        .collect();
    ipma_payload[association_count_position] += u8::try_from(added_count).unwrap();
    ipma_payload.splice(insert_position..insert_position, new_associations);
}

// Item data stored after the meta box moves by `growth` bytes.
fn shift_item_offsets(iloc_payload: &mut [u8], meta_end_offset: usize, growth: usize) {
    let version = iloc_payload[0];
    let offset_size = usize::from(iloc_payload[4] >> 4);
    let length_size = usize::from(iloc_payload[4] & 0x0f);
    let base_offset_size = usize::from(iloc_payload[5] >> 4);
    let index_size = if version >= 1 {
        usize::from(iloc_payload[5] & 0x0f)
    } else {
        0
    };
    let (item_count, mut position) = if version < 2 {
        (read_number(iloc_payload, 6, 2), 8)
    } else {
        (read_number(iloc_payload, 6, 4), 10)
    };

    for _ in 0..item_count {
        position += if version < 2 { 2 } else { 4 };
        let construction_method = if version >= 1 {
            let method = iloc_payload[position + 1] & 0x0f;
            position += 2;
            method
        } else {
            0
        };
        position += 2;
        let is_file_offset = construction_method == 0;
        let base_offset_position = position;
        let base_offset = read_number(iloc_payload, position, base_offset_size);
        position += base_offset_size;
        let extent_count = read_number(iloc_payload, position, 2);
        position += 2;

        if is_file_offset && base_offset_size > 0 && base_offset >= meta_end_offset {
            write_number(
                iloc_payload,
                base_offset_position,
                base_offset_size,
                base_offset + growth,
            );
        }
        for _ in 0..extent_count {
            position += index_size;
            let extent_offset = read_number(iloc_payload, position, offset_size);
            let shifts_extent =
                is_file_offset && base_offset_size == 0 && extent_offset >= meta_end_offset;
            if shifts_extent {
                write_number(iloc_payload, position, offset_size, extent_offset + growth);
            }
            position += offset_size + length_size;
        }
    }
}

fn read_number(bytes: &[u8], position: usize, length: usize) -> usize {
    let number = bytes[position..position + length]
        .iter()
        .fold(0, |total, byte| (total << 8) | usize::from(*byte));
    number
}

fn write_number(bytes: &mut [u8], position: usize, length: usize, number: usize) {
    for (index, byte) in bytes[position..position + length].iter_mut().enumerate() {
        let shift = 8 * (length - 1 - index);
        *byte = u8::try_from((number >> shift) & 0xff).unwrap();
    }
}

fn parse_boxes(bytes: &[u8]) -> Vec<RawBox> {
    let mut boxes = Vec::new();
    let mut position = 0;
    while position + BOX_HEADER_LENGTH <= bytes.len() {
        let box_size = read_number(bytes, position, 4);
        let box_type: [u8; 4] = bytes[position + 4..position + 8].try_into().unwrap();
        let payload = bytes[position + BOX_HEADER_LENGTH..position + box_size].to_vec();
        boxes.push(RawBox { box_type, payload });
        position += box_size;
    }
    boxes
}

fn make_box(box_type: [u8; 4], body: &[u8]) -> Vec<u8> {
    let raw_box = RawBox {
        box_type,
        payload: body.to_vec(),
    };
    join_boxes(&[raw_box])
}

fn join_boxes(boxes: &[RawBox]) -> Vec<u8> {
    let mut joined = Vec::new();
    for raw_box in boxes {
        let box_length = BOX_HEADER_LENGTH + raw_box.payload.len();
        joined.extend(u32::try_from(box_length).unwrap().to_be_bytes());
        joined.extend(raw_box.box_type);
        joined.extend(&raw_box.payload);
    }
    joined
}
