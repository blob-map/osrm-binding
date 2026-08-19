pub mod algorithm;
pub mod errors;
pub mod osrm_engine;
pub mod point;
pub mod route;
pub mod tables;
pub mod trip;
pub mod waypoints;
// src/lib.rs
use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;

#[repr(C)]
struct OsrmResult {
    code: i32,
    message: *mut c_char,
}

#[repr(C)]
struct OsrmSimpleRouteResult {
    code: i32,
    distance: f64,
    duration: f64,
    message: *mut c_char,
}

#[link(name = "osrm_wrapper", kind = "static")]
unsafe extern "C" {
    fn osrm_create(base_path: *const c_char, algorithm: *const c_char) -> *mut c_void;

    fn osrm_destroy(osrm_instance: *mut c_void);

    fn osrm_table(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        num_sources: usize,
        num_destinations: usize,
    ) -> OsrmResult;

    fn osrm_trip(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
    ) -> OsrmResult;

    fn osrm_route(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
    ) -> OsrmResult;

    fn osrm_simple_route(
        osrm_instance: *mut c_void,
        from_longitude: f64,
        from_latitude: f64,
        to_longitude: f64,
        to_latitude: f64,
    ) -> OsrmSimpleRouteResult;
    fn osrm_free_string(s: *mut c_char);
}

pub(crate) struct Osrm {
    instance: *mut c_void,
}

impl Osrm {
    pub(crate) fn new(base_path: &str, algorithm: &str) -> Result<Self, String> {
        let c_path = CString::new(base_path).map_err(|e| e.to_string())?;
        let c_algorithm = CString::new(algorithm).map_err(|e| e.to_string())?;
        let instance = unsafe { osrm_create(c_path.as_ptr(), c_algorithm.as_ptr()) };

        if instance.is_null() {
            Err("Failure to create an OSRM instance.".to_string())
        } else {
            Ok(Osrm { instance })
        }
    }

    pub(crate) fn trip(&self, coordinates: &[f64]) -> Result<String, String> {
        debug_assert_eq!(coordinates.len() % 2, 0);
        let result =
            unsafe { osrm_trip(self.instance, coordinates.as_ptr(), coordinates.len() / 2) };
        result_to_string(result)
    }

    pub(crate) fn route(&self, coordinates: &[f64]) -> Result<String, String> {
        debug_assert_eq!(coordinates.len() % 2, 0);
        let result =
            unsafe { osrm_route(self.instance, coordinates.as_ptr(), coordinates.len() / 2) };
        result_to_string(result)
    }

    pub(crate) fn simple_route(
        &self,
        from_longitude: f64,
        from_latitude: f64,
        to_longitude: f64,
        to_latitude: f64,
    ) -> Result<(f64, f64), String> {
        let result = unsafe {
            osrm_simple_route(
                self.instance,
                from_longitude,
                from_latitude,
                to_longitude,
                to_latitude,
            )
        };

        if result.code == 0 {
            return Ok((result.distance, result.duration));
        }

        Err(take_message(result.message)
            .map(|message| format!("OSRM error: {message}"))
            .unwrap_or_else(|error| error))
    }

    pub(crate) fn table(
        &self,
        coordinates: &[f64],
        num_sources: usize,
        num_destinations: usize,
    ) -> Result<String, String> {
        debug_assert_eq!(coordinates.len(), (num_sources + num_destinations) * 2);

        let result = unsafe {
            osrm_table(
                self.instance,
                coordinates.as_ptr(),
                num_sources + num_destinations,
                num_sources,
                num_destinations,
            )
        };
        result_to_string(result)
    }
}

fn result_to_string(result: OsrmResult) -> Result<String, String> {
    let message = take_message(result.message)?;
    if result.code == 0 {
        Ok(message)
    } else {
        Err(format!("OSRM error: {message}"))
    }
}

fn take_message(message_ptr: *mut c_char) -> Result<String, String> {
    if message_ptr.is_null() {
        return Err("OSRM returned a null message".to_string());
    }

    let message = unsafe { CStr::from_ptr(message_ptr) }
        .to_str()
        .map(str::to_owned)
        .map_err(|error| error.to_string());

    unsafe {
        osrm_free_string(message_ptr);
    }

    message
}

impl Drop for Osrm {
    fn drop(&mut self) {
        unsafe {
            osrm_destroy(self.instance);
        }
    }
}

unsafe impl Send for Osrm {}
unsafe impl Sync for Osrm {}
