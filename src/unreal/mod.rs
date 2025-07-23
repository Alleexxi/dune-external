use crate::unreal::{
    global::get_process, offsets::GOBJECTS, types::structs::FChunkedFixedUObjectArray,
};

// TODO: Add Unreal Functions
pub mod global;
pub mod offsets;
pub mod screen;
pub mod types;

pub fn update_gobjects() -> Result<(), String> {
    use crate::unreal::global::set_gobjects;
    use crate::unreal::types::structs::FUObjectItem;

    let proc = get_process();
    let gobjects = proc
        .read::<FChunkedFixedUObjectArray>(0x140000000 + GOBJECTS)
        .expect(""); //TODO: Change OFFSET HANDLING??

    println!("GObjects {:?}", gobjects);

    // Validate gobjects structure before proceeding
    if gobjects.objects.is_null() {
        return Err("gobjects.objects is null pointer".to_string());
    }

    if gobjects.num_chunks == 0 {
        return Err("gobjects.num_chunks is zero".to_string());
    }

    let mut object_items: Vec<FUObjectItem> = Vec::new();

    // Handle chunked case (UE 4.20 and later)
    let num_elements_per_chunk = 64 * 1024; // 64K elements per chunk

    println!(
        "Processing {} chunks with {} total elements",
        gobjects.num_chunks, gobjects.num_elements
    );

    // Iterate through each chunk
    for i in 0..gobjects.num_chunks {
        // Read the chunk pointer from the objects array
        let chunk_ptr_addr = (gobjects.objects as usize) + (i as usize * 0x8);
        let chunk_start = proc.read::<usize>(chunk_ptr_addr).map_err(|e| {
            format!(
                "Failed to read chunk pointer at 0x{:x}: {}",
                chunk_ptr_addr, e
            )
        })?;

        if chunk_start == 0 {
            println!("Chunk {} has null pointer, skipping", i);
            continue;
        }

        // Calculate how many elements to read from this chunk
        let remaining_elements =
            (gobjects.num_elements as usize).saturating_sub(object_items.len());
        let elements_in_this_chunk = std::cmp::min(remaining_elements, num_elements_per_chunk);

        // Read FUObjectItem structs directly from this chunk
        for j in 0..elements_in_this_chunk {
            let item_addr = chunk_start + (j * 24); // Each FUObjectItem is 24 bytes

            match proc.read::<FUObjectItem>(item_addr) {
                Ok(item) => {
                    object_items.push(item);
                }
                Err(e) => {
                    println!("Failed to read FUObjectItem at 0x{:x}: {}", item_addr, e);
                    // Continue reading other items instead of failing completely
                }
            }
        }

        // Break if we've read all elements
        if object_items.len() >= gobjects.num_elements as usize {
            break;
        }
    }
    set_gobjects(object_items);

    Ok(())
}
