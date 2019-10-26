use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr::NonNull;
use std::path::Path;

const PAYLOAD_FLOAT: u32 = akumuli_sys::aku_PData_PARAMID_BIT | akumuli_sys::aku_PData_TIMESTAMP_BIT | akumuli_sys::aku_PData_FLOAT_BIT;

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
        akumuli_sys::aku_LogLevel_AKU_LOG_TRACE => log::trace!("{}", msg),
        akumuli_sys::aku_LogLevel_AKU_LOG_INFO => log::info!("{}", msg),
        akumuli_sys::aku_LogLevel_AKU_LOG_ERROR => log::error!("{}", msg),
        _ => log::info!("{}", msg),
    }
}

fn initialize() {
    unsafe {
        akumuli_sys::aku_initialize(Some(panic_handler), Some(logger));
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

pub struct DB(NonNull<akumuli_sys::aku_Database>);

impl DB {
    pub fn open(path: &str, suffix: &str) -> Result<Self, &'static str> {
        initialize();

        let params = akumuli_sys::aku_FineTuneParams {
            logger: Some(logger),
            input_log_volume_size: 1000,
            input_log_volume_numb: 1,
            input_log_concurrency: 0x1000000,
            input_log_path: CString::new("log.log").unwrap().as_ptr()
        };

        let absolute_path = Path::new(path).join(suffix);
        let absolute_path = CString::new(absolute_path.to_str().unwrap()).unwrap();

        let ptr = unsafe {
            akumuli_sys::aku_open_database(
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
            akumuli_sys::aku_create_database_ex(
                suffix.as_ptr(), path_str.as_ptr(), path_str.as_ptr(),
                config.num_volumes, config.page_size, config.allocate
            )
        };

        if result != akumuli_sys::APR_SUCCESS {
            return Err("Failed to create database");
        }

        Self::open(path, config.suffix)
    }

    pub fn create_session(&self) -> Option<Session> {
        let session = unsafe {
            akumuli_sys::aku_create_session(
                self.0.as_ptr()
            )
        };

        Some(Session(NonNull::new(session)?))
    }
}

pub struct Session(NonNull<akumuli_sys::aku_Session>);

impl Session {
    pub fn metric_to_param_id(&self, metric: &str) -> Result<u64, String> {
        let len = metric.len();
        let metric = CString::new(metric).unwrap();

        let mut sample = default_sample();

        println!("SIGBUS occurs here >:^(");

        let success = unsafe {
            akumuli_sys::aku_series_to_param_id(
                self.0.as_ptr(), metric.as_ptr(), metric.as_ptr().add(len),
                (&mut sample) as *mut akumuli_sys::aku_Sample
            )
        };

        println!("See?");

        if success == akumuli_sys::APR_SUCCESS {
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
            akumuli_sys::aku_write(
                self.0.as_ptr(), (&sample) as *const akumuli_sys::aku_Sample
            )
        };

        if success == akumuli_sys::APR_SUCCESS {
            Ok(())
        } else {
            Err(format!("aku_write returned {}", success))
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            akumuli_sys::aku_destroy_session(self.0.as_ptr());
        }
    }
}

fn default_sample() -> akumuli_sys::aku_Sample {
    akumuli_sys::aku_Sample {
        timestamp: 0,
        paramid: 0,
        payload: akumuli_sys::aku_PData {
            float64: 0.0,
            size: std::mem::size_of::<akumuli_sys::aku_Sample>() as u16,
            type_: PAYLOAD_FLOAT as u16,
            data: akumuli_sys::__IncompleteArrayField::new()
        }
    }
}