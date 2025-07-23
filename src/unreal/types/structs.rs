use memflex::external::OwnedProcess;
use offsetter::offset_debug;

use crate::unreal::global::STRING_CACHE;
use crate::unreal::global::get_gnames;
use crate::unreal::global::get_process;
use crate::unreal::types::enums::{EClassCastFlags, EClassFlags, EObjectFlags, EPropertyFlags};

// Unreal Engine 5....
// #[flamer::flame]
#[flamer::flame]
#[derive(Debug, Clone, Copy)]
pub struct FName {
    pub comparison_index: u32,
    pub _number: u32,
}

impl FName {
    #[flamer::flame]
    pub fn to_string(&self) -> String {
        let index = self.comparison_index as usize;
        let mut cache = STRING_CACHE.lock().unwrap();

        if index >= cache.len() {
            cache.resize_with(index + 1, || None);
        }

        match &cache[index] {
            Some(s) => return s.clone(),
            _ => (),
        }

        if self.comparison_index == 0 {
            return "None".to_string();
        }

        let proc = get_process();
        let gnames_address = get_gnames();
        const NAME_SIZE: usize = 1024;

        let chunk_offset = (self.comparison_index >> 16) as u32;
        let name_offset = self.comparison_index as u16;

        let chunk_ptr_address = gnames_address + 8 * (chunk_offset as usize + 2);

        let name_pool_chunk_ptr = match proc.read::<u64>(chunk_ptr_address) {
            Ok(ptr) if ptr != 0 => ptr as usize,
            _ => return "None".to_string(),
        };

        let name_pool_chunk = name_pool_chunk_ptr + 2 * name_offset as usize;
        let name_length_raw = match proc.read::<u16>(name_pool_chunk) {
            Ok(len) if len != 0 => len,
            _ => return "None".to_string(),
        };

        let name_length = (name_length_raw >> 6) as usize;
        if name_length == 0 {
            return "None".to_string();
        }

        let safe_length = name_length.min(NAME_SIZE);
        let mut name_bytes = vec![0u8; safe_length];

        // If read_buf fails, return "None"
        if proc.read_buf(name_pool_chunk + 2, &mut name_bytes).is_err() {
            return "None".to_string();
        }

        let name_string = String::from_utf8_lossy(&name_bytes)
            .trim_end_matches('\0')
            .to_string();

        if name_string.is_empty() || name_string == "None".to_string() {
            "None".to_string()
        } else {
            cache[index] = Some(name_string.clone());
            name_string
        }
    }
}

// TArray - Unreal Engine's dynamic array
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct TArray<T> {
    pub data: usize, // T* Data
    pub count: i32,  // int32_t Count
    pub max: i32,    // int32_t Max
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy + 'static> TArray<T> {
    pub fn num(&self) -> i32 {
        self.count
    }

    pub fn is_valid_index(&self, index: i32) -> bool {
        index >= 0 && index < self.num()
    }

    pub fn slack(&self) -> i32 {
        self.max - self.count
    }

    pub fn is_valid(&self) -> bool {
        self.data != 0 && self.count >= 0 && self.count <= self.max
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get_data<U>(&self, index: i32) -> usize
    where
        U: Copy + 'static,
    {
        if !self.is_valid_index(index) {
            return 0;
        }
        // Get Type of _marker and use that element size
        let element_size = std::mem::size_of::<U>();
        let element_address = self.data + (index as usize * element_size);
        element_address
    }

    pub fn get<U>(&self, index: i32) -> Result<U, Box<dyn std::error::Error>>
    where
        U: Copy + 'static,
    {
        let proc = get_process();
        if !self.is_valid_index(index) {
            return Err(format!("Index {} out of bounds (count: {})", index, self.count).into());
        }

        let element_size = std::mem::size_of::<U>();
        let element_address = self.data + (index as usize * element_size);
        Ok(proc.read::<U>(element_address)?)
    }
    #[flamer::flame]
    pub fn read_all<U>(&self) -> Result<Vec<U>, Box<dyn std::error::Error>>
    where
        U: Copy + 'static,
    {
        let proc = get_process();

        if !self.is_valid() || self.is_empty() {
            return Ok(Vec::new());
        }

        let element_size = std::mem::size_of::<U>();
        let total_size = self.count as usize * element_size;
        let mut buffer = vec![0u8; total_size];

        proc.read_buf(self.data, &mut buffer)?;

        let ptr = buffer.as_ptr() as *const U;
        let slice = unsafe { std::slice::from_raw_parts(ptr, self.count as usize) };
        Ok(slice.to_vec())
    }

    pub fn for_each<U, F>(&self, mut func: F) -> Result<(), Box<dyn std::error::Error>>
    where
        U: Copy + 'static,
        F: FnMut(i32, U),
    {
        if !self.is_valid() || self.is_empty() {
            return Ok(());
        }
        let proc = get_process();

        let element_size = std::mem::size_of::<U>();
        let total_size = self.count as usize * element_size;
        let mut buffer = vec![0u8; total_size];

        proc.read_buf(self.data, &mut buffer)?;

        let ptr = buffer.as_ptr() as *const U;
        let slice = unsafe { std::slice::from_raw_parts(ptr, self.count as usize) };

        for (i, &item) in slice.iter().enumerate() {
            func(i as i32, item);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FUObjectItem {
    pub object: usize,
    pub flags: i32,
    pub cluster_root_index: i32,
    pub serial_number: i32,
}

impl FUObjectItem {
    pub fn as_uobject(&self) -> UObject {
        let proc = get_process();
        proc.read::<UObject>(self.object).expect("Error.")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FChunkedFixedUObjectArray {
    pub objects: *mut *mut FUObjectItem,
    pub pre_allocated_objects: *mut FUObjectItem,
    pub max_elements: i32,
    pub num_elements: i32,
    pub max_chunks: i32,
    pub num_chunks: i32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UObject {
    pub object_ptr: usize,
    pub flags: EObjectFlags,
    pub index: i32,
    pub _pad: u64,
    pub name: FName,
    pub outer: *mut UObject,
    pad_28: [u8; 8],
}

impl UObject {
    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_fullname(&self) -> String {
        let name = self.get_name();
        let proc = get_process();
        let mut temp = String::new();
        let mut outer = self.outer;

        while !outer.is_null() {
            let outer_obj: UObject = proc.read(outer as usize).unwrap();
            let outer_name = outer_obj.get_name();
            temp = if temp.is_empty() {
                format!("{}.", outer_name)
            } else {
                format!("{}.{temp}", outer_name)
            };
            outer = outer_obj.outer;
        }

        let fullname = format!("{}{}", temp, name);
        fullname
    }
}

offset_debug! {
    pub struct AActor {
        0x18 pub name: FName,
        0x240 pub root: usize,
    }
}

// Implement GetName and so on...

// Impl FProperty
// UProperty
// UClass

#[derive(Clone, Copy)]
pub union FFieldObjectUnion {
    pub field: *mut FField,
    pub object: *mut UObject,
}

impl std::fmt::Debug for FFieldObjectUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FFieldVariant {
    pub container: FFieldObjectUnion,
    pub b_is_u_object: bool,
}

//////////////

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FFieldClass {
    pub name: FName,
    pub id: u64,
    pub castflags: EClassCastFlags,
    pub classflags: EClassFlags,
    pub superclass: *mut FFieldClass,
    pub defaultobject: *mut FField,
}

///////////////

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FField {
    pub object_ptr: usize,
    pub class_private: FFieldClass, // TODO!!!!
    pub owner: FFieldVariant,
    pub next: *mut FField,
    pub name_private: FName,
    pub flags_private: EObjectFlags,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FProperty {
    pub base: FField,
    pub arraydim: u32,
    pub elementsize: u32,
    pub property_flags: EPropertyFlags,
    pub repindex: u16,
    pub blueprintreplicationcondition: u8,
    pub offset: i32,
    pub rep_notify_func: FName,
    pub property_link_next: *mut FProperty,
    pub next_ref: *mut FProperty,
    pub destructor_link_next: *mut FProperty,
    pub post_construct_link_next: *mut FProperty,
}

///
///
#[derive(Debug, Clone, Copy)]
pub struct FMinimalViewInfo {
    pub location: FVector,
    pub rotation: FVector,
    pub fov: f32,
}

// FVector

#[derive(Debug, Clone, Copy)]
pub struct FVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl FVector {
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn distance(self, other: FVector) -> f64 {
        (self - other).magnitude()
    }
    #[flamer::flame]
    pub fn dot(&self, other: &FVector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &FVector) -> FVector {
        FVector {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn normalize(&self) -> FVector {
        let mag = self.magnitude();
        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
    #[flamer::flame]
    // Add to_matrix
    pub fn to_matrix(&self) -> [[f64; 4]; 4] {
        // Skidded.
        let origin = FVector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let rad_pitch = self.x * std::f64::consts::PI / 180.0;
        let rad_yaw = self.y * std::f64::consts::PI / 180.0;
        let rad_roll = self.z * std::f64::consts::PI / 180.0;

        let sp = rad_pitch.sin();
        let cp = rad_pitch.cos();
        let sy = rad_yaw.sin();
        let cy = rad_yaw.cos();
        let sr = rad_roll.sin();
        let cr = rad_roll.cos();

        [
            [cp * cy, cp * sy, sp, 0.0],
            [
                sr * sp * cy - cr * sy,
                sr * sp * sy + cr * cy,
                -sr * cp,
                0.0,
            ],
            [
                -(cr * sp * cy + sr * sy),
                cy * sr - cr * sp * sy,
                cr * cp,
                0.0,
            ],
            [origin.x, origin.y, origin.z, 1.0],
        ]
    }
}

impl std::ops::Sub for FVector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Div for FVector {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl std::ops::Add for FVector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Neg for FVector {
    type Output = FVector;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// TMap stuff
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TPair<K, V> {
    pub key: K,
    pub value: V,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TSetElement<T> {
    pub value: T,
    pub hash_next_id: i32,
    pub hash_index: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct TMap<K, V> {
    pub data: TArray<TSetElement<TPair<K, V>>>,
    pub unknown_data_01: [u8; 4],
    pub unknown_data_02: [u8; 4],
    pub unknown_data_03: [u8; 8],
    pub unknown_data_04: [u8; 8],
    pub unknown_data_maybe_size: [u8; 4],
    pub unknown_data_maybe_flag: [u8; 4],
    pub unknown_data_05: [u8; 8],
    pub unknown_data_06: [u8; 8],
    pub unknown_data_07: [u8; 8],
    pub unknown_data_maybe_size02: [u8; 4],
    pub unknown_data_08: [u8; 4],
}

impl<K: Copy + 'static, V: Copy + 'static> TMap<K, V> {
    /// Reads all key-value pairs from the TMap into a Vec<(K, V)>
    pub fn read_all(&self) -> Result<Vec<(K, V)>, Box<dyn std::error::Error>> {
        self.data
            .read_all::<TSetElement<TPair<K, V>>>()
            .map(|elements| {
                elements
                    .into_iter()
                    .map(|el| {
                        let pair = el.value;
                        (pair.key, pair.value)
                    })
                    .collect()
            })
    }

    /// Iterates over all key-value pairs using a closure
    pub fn for_each<F>(&self, mut func: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(i32, K, V),
    {
        self.data.for_each::<TSetElement<TPair<K, V>>, _>(|i, el| {
            let pair = el.value;
            func(i, pair.key, pair.value);
        })
    }
}
