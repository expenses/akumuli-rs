use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::path::Path;

use akumuli_sys::{
    aku_PData_PARAMID_BIT, aku_PData_TIMESTAMP_BIT, aku_PData_FLOAT_BIT,
    aku_LogLevel_AKU_LOG_TRACE, aku_LogLevel_AKU_LOG_INFO, aku_LogLevel_AKU_LOG_ERROR,
    APR_SUCCESS,
    aku_initialize, aku_open_database, aku_create_database_ex, aku_create_session, aku_write,
    aku_series_to_param_id, aku_destroy_session,
    aku_Database, aku_FineTuneParams, aku_Session, aku_Sample, aku_PData
};

const PAYLOAD_FLOAT: u32 = aku_PData_PARAMID_BIT | aku_PData_TIMESTAMP_BIT | aku_PData_FLOAT_BIT;

extern "C" fn panic_handler(msg: *const c_char) {
    let msg = unsafe {
        CStr::from_ptr(msg)
    }.to_string_lossy();

    panic!("{}", msg);
}

extern "C" fn logger(log_level: u32, msg: *const c_char) {
    let msg = unsafe {
        CStr::from_ptr(msg)
    }.to_string_lossy();

    match log_level {
        aku_LogLevel_AKU_LOG_TRACE => log::trace!("{}", msg),
        aku_LogLevel_AKU_LOG_INFO => log::info!("{}", msg),
        aku_LogLevel_AKU_LOG_ERROR => log::error!("{}", msg),
        _ => log::info!("{}", msg),
    }
}

fn initialize() {
    unsafe {
        aku_initialize(Some(panic_handler), Some(logger));
    }
}

pub struct DBConfig<'a> {
    pub num_volumes: i32,
    pub allocate: bool,
    pub page_size: u64,
    pub suffix: &'a str
}

impl Default for DBConfig<'_> {
    fn default() -> Self {
        Self {
            num_volumes: 1,
            allocate: true,
            page_size: 4096 * 1024 * 1024,
            suffix: "db"
        }
    }
}

pub struct DB(NonNull<aku_Database>);

impl DB {
    pub fn open(path: &str, suffix: &str) -> Result<Self, &'static str> {
        initialize();

        let log = CString::new("log.log").unwrap();

        let params = aku_FineTuneParams {
            logger: Some(logger),
            input_log_volume_size: 1000,
            input_log_volume_numb: 1,
            input_log_concurrency: 0x0100_0000,
            input_log_path: log.as_ptr()
        };

        let absolute_path = Path::new(path).join(suffix);
        let absolute_path = CString::new(absolute_path.to_str().unwrap()).unwrap();

        let ptr = unsafe {
            aku_open_database(
                absolute_path.as_ptr(), params
            )
        };

        match NonNull::new(ptr) {
            Some(ptr) => Ok(DB(ptr)),
            None => Err("Failed to open database")
        }
    }

    pub fn open_or_create(path: &str, config: &DBConfig<'_>) -> Result<Self, &'static str> {
        if Path::new(path).exists() {
            Self::open(path, config.suffix)
        } else {
            Self::create(path, config)
        }
    }

    pub fn create(path: &str, config: &DBConfig<'_>) -> Result<Self, &'static str> {
        initialize();

        let suffix = CString::new(config.suffix).map_err(|_| "Failed to read suffix")?;
        let path_str = CString::new(path).unwrap();

        let result = unsafe {
            aku_create_database_ex(
                suffix.as_ptr(), path_str.as_ptr(), path_str.as_ptr(),
                config.num_volumes, config.page_size, config.allocate
            )
        };

        if result != APR_SUCCESS {
            return Err("Failed to create database");
        }

        Self::open(path, config.suffix)
    }

    pub fn create_session(&self) -> Option<Session> {
        let session = unsafe {
            aku_create_session(
                self.0.as_ptr()
            )
        };

        Some(Session(NonNull::new(session)?))
    }
}

pub struct Session(NonNull<aku_Session>);

impl Session {
    pub fn metric_to_param_id(&self, metric: &str) -> Result<u64, String> {
        let len = metric.len();
        let metric = CString::new(metric).unwrap();

        let mut sample = default_sample();

        println!("SIGBUS occurs here >:^(");

        let success = unsafe {
            aku_series_to_param_id(
                self.0.as_ptr(), metric.as_ptr(), metric.as_ptr().add(len),
                (&mut sample) as *mut aku_Sample
            )
        };

        println!("See?");

        if success == APR_SUCCESS {
            Ok(sample.paramid)
        } else {
            Err(format!("aku_series_to_param_id returned {}", success))
        }
    }

    pub fn write(&self, metric: &str, data: f64) -> Result<(), String> {
        let param_id = self.metric_to_param_id(metric)?;

        let mut sample = default_sample();
        sample.timestamp = chrono::Utc::now().timestamp() as u64;
        sample.paramid = param_id;
        sample.payload.float64 = data;

        let success = unsafe {
            aku_write(
                self.0.as_ptr(), (&sample) as *const aku_Sample
            )
        };

        if success == APR_SUCCESS {
            Ok(())
        } else {
            Err(format!("aku_write returned {}", success))
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            aku_destroy_session(self.0.as_ptr());
        }
    }
}

fn default_sample() -> aku_Sample {
    aku_Sample {
        timestamp: 0,
        paramid: 0,
        payload: aku_PData {
            float64: 0.0,
            size: std::mem::size_of::<aku_Sample>() as u16,
            type_: PAYLOAD_FLOAT as u16,
            data: akumuli_sys::__IncompleteArrayField::new()
        }
    }
}