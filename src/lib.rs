use std::{io::{Write, Cursor, Read}, collections::HashMap};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const WEIRD_CHAR: u8 = b'q';
const CONSECUTIVE_WEIRD_CHARS: usize = 1024;
const WEIRD_CHAR_1KB: [u8; CONSECUTIVE_WEIRD_CHARS] = [WEIRD_CHAR; CONSECUTIVE_WEIRD_CHARS];

pub struct DataToWrite {
    pub key: String,
    pub data: Vec<u8>,
}


/// Like `generate_included_data_file`, but writes data to a writer, rather
/// than a file
pub fn generate_included_data(
    mut writer: impl Write,
    num_kb: usize,
) -> Result<(), String> {
    if num_kb == 0 {
        return Err("generate_included_data_file received 0kb to write. Not writing an empty file...".into());
    }

    for _ in 0..num_kb {
        writer.write_all(&WEIRD_CHAR_1KB[..])
            .map_err(|e| format!("Error writing 1kb chunk to file: {e}"))?;
    }

    Ok(())
}

/// provide the name of the file that this compile-time
/// function will write to (ie: put this in your build.rs)
/// For example, you can provide an out_path_name of "myincludeddata.txt"
/// and then in your code, you can have somehwere `include_bytes!("myincludeddata.txt")
/// which will then cause the compiled binary to include all the data.
/// you must also specify `num_kb` which is the numbers of kilobytes of data
/// to be generated. minimum is 1kb. function errors if 0 is provided.
/// For example, if `num_kb = 10`, this will write 10 * 1024 = 10kb = 10240 bytes
/// in theory there's no max... but just be careful.
pub fn generate_included_data_file(
    out_path_name: &str,
    num_kb: usize,
) -> Result<(), String> {
    let mut out_file = std::fs::File::open(out_path_name)
        .map_err(|e| format!("Error opening file: {e}"))?;
    generate_included_data(&mut out_file, num_kb)
}

/// returns 2 usizes, the first is the byte index of where
/// the weird data starts.
/// the second is the total size of the weird data section.
pub fn get_weird_indices(data: &[u8]) -> Option<(usize, usize)> {
    let mut weird_index_start = None;
    let mut weird_data_found = None;
    for (index, byte) in data.iter().enumerate() {
        if *byte == WEIRD_CHAR {
            if weird_index_start.is_none() {
                weird_index_start = Some(index);
            }
            if let Some(i) = weird_index_start {
                let num_weird_bytes = index - i;
                if num_weird_bytes >= CONSECUTIVE_WEIRD_CHARS {
                    weird_data_found = Some(i);
                }
            }
        } else {
            if let Some(i) = weird_data_found {
                let num_weird_bytes = index - i;
                return Some((i, num_weird_bytes));
            }
            weird_data_found = None;
            weird_index_start = None;
        }
    }
    if let Some(i) = weird_data_found {
        let current_index = data.len() - 1;
        let num_weird_bytes = current_index - i + 1;
        return Some((i, num_weird_bytes));
    }
    None
}

/// returns the number of bytes necessary to store the write data
/// ie: this is the number of bytes of all the data + headers
pub fn get_data_write_required_len(write_data: &Vec<DataToWrite>) -> usize {
    let mut total_write_data = 0;
    for data_item in write_data.iter() {
        // 6 bytes for the header,
        // 4 for the size of the data,
        // 2 for the size of the key
        total_write_data += 6;
        total_write_data += data_item.key.as_bytes().len();
        total_write_data += data_item.data.len();
    }
    total_write_data
}

pub fn write_to_included_section(
    included: &mut Vec<u8>,
    mut write_data: Vec<DataToWrite>,
) -> Result<(), String> {
    let (mut write_index, weird_data_len) = get_weird_indices(&included[..])
        .ok_or("Failed to find indices of included data")?;

    let mut write_to_included = |byte: u8| -> Result<(), String> {
        if let Some(write_byte) = included.get_mut(write_index) {
            *write_byte = byte;
        } else {
            return Err(format!("Failed to write to included data section at position {}", write_index));
        }
        write_index += 1;
        Ok(())
    };

    let total_write_data = get_data_write_required_len(&write_data);
    // +4 because we require an initial header of 4 bytes to represent
    // how much data is being stored in total
    if total_write_data + 4 > weird_data_len {
        return Err(format!("Attempting to write {} bytes but included data section only has {} available", total_write_data, weird_data_len));
    }
    // write this initial header of 4 bytes:
    let mut header = vec![];
    let header_size = write_data.len();
    header.write_u32::<BigEndian>(header_size as u32)
        .map_err(|e| format!("Failed to write initial header: {e}"))?;
    if header.len() != 4 {
        return Err(format!("Failed to write initial header: Expected 4 bytes, but read {}", header.len()));
    }
    for byte in header {
        write_to_included(byte)?;
    }
    for data_item in write_data.drain(..) {
        // write the header of the key length
        let mut key_header = vec![];
        let key_size = data_item.key.as_bytes().len();
        key_header.write_u16::<BigEndian>(key_size as u16)  
            .map_err(|e| format!("Failed to write key size header: {e}"))?;
        if key_header.len() != 2 {
            return Err(format!("Failed to write key size header: Expected 2 bytes, but read {}", key_header.len()));
        }
        for byte in key_header {
            write_to_included(byte)?;
        }
        // write the actual key:
        for byte in data_item.key.as_bytes() {
            write_to_included(*byte)?;
        }

        // write the header of the data length
        let mut data_header = vec![];
        let data_size = data_item.data.len();
        data_header.write_u32::<BigEndian>(data_size as u32)
            .map_err(|e| format!("Failed to write data size header: {e}"))?;
        if data_header.len() != 4 {
            return Err(format!("Failed to write data size header: Expected 4 bytes, but read {}", data_header.len()));
        }
        for byte in data_header {
            write_to_included(byte)?;
        }
        // write the actual data:
        for byte in data_item.data.iter() {
            write_to_included(*byte)?;
        }
    }

    Ok(())
}

/// call this from within your actual compiled program and pass in
/// the included data section (that was modified externally)
/// this fn will iter that data and call a callback each time it finds a key + data
/// callback should return false if you want to stop iteration
pub fn iter_data_section<Cb: FnMut(usize, String, &[u8]) -> bool>(
    data: &[u8],
    mut cb: Cb,
) -> Result<(), String> {
    let mut cursor = Cursor::new(data);
    // read the initial size header:
    let total_data_items = cursor.read_u32::<BigEndian>()
        .map_err(|e| format!("Failed to read initial header of size 4: {}", e))?;
    let total_data_items = total_data_items as usize;
    for i in 0..total_data_items {
        // read key header:
        let key_header_size = cursor.read_u16::<BigEndian>()
            .map_err(|e| format!("Failed to read key header of size 2: {}", e))?;
        let key_header_size = key_header_size as usize;
        let mut key_buf = Vec::with_capacity(key_header_size);
        unsafe { key_buf.set_len(key_header_size) }
        cursor.read_exact(&mut key_buf)
            .map_err(|e| format!("Failed to read key of size {}: {}", key_header_size, e))?;
        let key = String::from_utf8(key_buf)
            .map_err(|e| format!("Failed to read key as string: {}", e))?;
        // read data header:
        let data_header_size = cursor.read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read data header of size 4: {}", e))?;
        let data_header_size = data_header_size as usize;
        let cursor_pos = cursor.position() as usize;
        let data_ends_at = cursor_pos + data_header_size;
        if let Some(data_piece) = data.get(cursor_pos..data_ends_at) {
            if cb(i, key, data_piece) {
                break;
            }
        }
        cursor.set_position(data_ends_at as u64);
    }
    Ok(())
}

/// given the included data section (ie: this is the actual array of data
/// within the compiled file, not the file itself), return the data section if a section
/// is found corresponding to the provided key, otherwise return None
pub fn get_data_section_by_key(
    key: &str,
    data: &[u8]
) -> Option<Vec<u8>> {
    let mut ret = None;
    let _ = iter_data_section(data, |_, key_in_data, data_section| {
        if key_in_data == key {
            ret = Some(data_section.to_vec());
            true
        } else {
            false
        }
    });
    ret
}

/// given the data section (the actual data array in the compiled file, not
/// the file itself), return a hashmap of all data with their keys
pub fn get_all_data_sections(
    data: &[u8]
) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut out = HashMap::new();
    iter_data_section(data, |_, key, data_section| {
        out.insert(key, data_section.to_vec());
        false
    })?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn weird_indices_work1() {
        let mut out: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut out);
        generate_included_data(&mut cursor, 2).expect("Failed to generate included data?");
        let mut data = vec![1, 2, 3];
        data.extend(out);
        data.push(4);
        assert_eq!(data.len(), 2048 + 3 + 1);
        let (weird_start, weird_len) = get_weird_indices(&data).expect("Failed to find weird indices");
        assert_eq!(weird_start, 3);
        assert_eq!(weird_len, 2048);
    }

    #[test]
    fn weird_indices_work2() {
        let mut out: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut out);
        generate_included_data(&mut cursor, 2).expect("Failed to generate included data?");
        let mut data = vec![1, 2, 3];
        data.extend(out);
        assert_eq!(data.len(), 2048 + 3);
        let (weird_start, weird_len) = get_weird_indices(&data).expect("Failed to find weird indices");
        assert_eq!(weird_start, 3);
        assert_eq!(weird_len, 2048);
    }

    #[test]
    fn read_write_works() {
        let mut out: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut out);
        generate_included_data(&mut cursor, 2).expect("Failed to generate included data?");
        let mut data = vec![1, 2, 3];
        data.extend(out);
        data.push(4);
        let expected_data = vec![100, 101, 102, 103, 104, 105];
        let write_items = vec![
            DataToWrite { key: "hello".to_string(), data: expected_data.clone() }
        ];
        // actual data:
        write_to_included_section(&mut data, write_items).expect("Failed to write to included section");
        let actual_data = data.get(3..2048 + 3).expect("Failed to get actual data section");
        let got_data = get_data_section_by_key("hello", &actual_data[..]).expect("Failed to get hello key from included data");
        assert_eq!(got_data, expected_data);
    }

    #[test]
    fn read_write_works2() {
        let mut out: Vec<u8> = vec![];
        let mut cursor = Cursor::new(&mut out);
        generate_included_data(&mut cursor, 2).expect("Failed to generate included data?");
        let mut data = vec![1, 2, 3];
        data.extend(out);
        data.push(4);
        let expected_data1 = vec![100, 101, 102, 103, 104, 105];
        let expected_data2 = vec![200, 201, 202, 203, 204, 205];
        let expected_data3 = vec![33, 34, 35];
        let write_items = vec![
            DataToWrite { key: "hello".to_string(), data: expected_data1.clone() },
            DataToWrite { key: "xyz".to_string(), data: expected_data2.clone() },
            DataToWrite { key: "abc".to_string(), data: expected_data3.clone() },
        ];
        // actual data:
        write_to_included_section(&mut data, write_items).expect("Failed to write to included section");
        let actual_data = data.get(3..2048 + 3).expect("Failed to get actual data section");
        let all_data_map = get_all_data_sections(&actual_data[..]).expect("Failed to get all data map");
        let data1 = all_data_map.get("hello").expect("Failed to get hello data section");
        assert_eq!(*data1, expected_data1);
        let data2 = all_data_map.get("xyz").expect("Failed to get xyz data section");
        assert_eq!(*data2, expected_data2);
        let data3 = all_data_map.get("abc").expect("Failed to get abc data section");
        assert_eq!(*data3, expected_data3);
        assert_eq!(all_data_map.len(), 3);
    }
}
