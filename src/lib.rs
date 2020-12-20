use std::error::Error;
use std::ffi::CString;

mod sys {
    use std::sync::RwLock;

    use lazy_static::lazy_static;

    lazy_static! {
        pub(crate) static ref CLIENTS: RwLock<Vec<Box<dyn Fn(&[&[f32]], &mut [&mut [f32]], usize) -> () + Send + Sync>>> =
            RwLock::new(vec![]);
    }

    pub(crate) extern "C" fn callback(
        ctx: i32,
        inputs: *const *const f32,
        num_inputs: i32,
        outputs: *mut *mut f32,
        num_outputs: i32,
        num_samples: i32,
    ) -> () {
        unsafe {
            let num_inputs = num_inputs as usize;
            let num_outputs = num_outputs as usize;
            let num_samples = num_samples as usize;

            let mut inputs_as_slices: [&[f32]; 128] = std::mem::zeroed();
            for i in 0..num_inputs {
                inputs_as_slices[i] =
                    std::slice::from_raw_parts(*inputs.offset(i as isize), num_samples);
            }
            let mut outputs_as_slices: [&mut [f32]; 128] = std::mem::zeroed();
            for i in 0..num_outputs {
                outputs_as_slices[i] =
                    std::slice::from_raw_parts_mut(*outputs.offset(i as isize), num_samples);
            }

            CLIENTS.read().expect("read")[ctx as usize](
                &inputs_as_slices[..num_inputs],
                &mut outputs_as_slices[..num_outputs],
                num_samples,
            );
        }
    }

    #[link(name = "JuceRustBindings")]
    extern "C" {
        pub(crate) fn get_devices() -> usize;
        pub(crate) fn stop_devices() -> ();
        pub(crate) fn activate_device(
            driver: *const libc::c_char,
            input_name: *const libc::c_char,
            output_name: *const libc::c_char,
            input_channels: i32,
            output_channels: i32,
            sample_rate: f64,
            buffer_size: i32,
            target: i32,
            callback: extern "C" fn(i32, *const *const f32, i32, *mut *mut f32, i32, i32),
        ) -> i32;
    }
}

const ERR_NO_DRIVER: i32 = -1;
const ERR_NO_DEVICE: i32 = -2;
const ERR_MGR_CONSTRUCT: i32 = -3;
const ERR_DEV_CONSTRUCT: i32 = -4;

fn get_devices() -> usize {
    unsafe { sys::get_devices() }
}

pub fn stop_devices() -> () {
    unsafe { sys::stop_devices() }
    sys::CLIENTS.write().expect("write").clear();
}

pub fn activate_device(
    driver: &str,
    input_name: &str,
    output_name: &str,
    input_channels: usize,
    output_channels: usize,
    sample_rate: usize,
    buffer_size: usize,
    f: Box<dyn Fn(&[&[f32]], &mut [&mut [f32]], usize) -> () + Send + Sync>,
) -> Result<i32, Box<dyn Error>> {
    let id = {
        let mut clients = sys::CLIENTS.write().expect("write");
        clients.push(f);
        clients.len() - 1
    };

    match unsafe {
        sys::activate_device(
            CString::new(driver)?.as_ptr(),
            CString::new(input_name)?.as_ptr(),
            CString::new(output_name)?.as_ptr(),
            input_channels as i32,
            output_channels as i32,
            sample_rate as f64,
            buffer_size as i32,
            id as i32,
            sys::callback,
        )
    } {
        ERR_NO_DRIVER => Err(format!("Driver {} could not be loaded", driver).into()),
        ERR_NO_DEVICE => Err(format!(
            "Input device {} or output device {} could not be created",
            input_name, output_name
        )
        .into()),
        ERR_MGR_CONSTRUCT => Err("Device manager failed to construct".into()),
        ERR_DEV_CONSTRUCT => Err(format!(
            "Input device {} or output device {} could not be constructed",
            input_name, output_name
        )
        .into()),
        _ => Ok(i),
    }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::time::Duration;

    use super::*;

    #[test]
    fn smoke_test() {
        println!("we got {} devices", get_devices());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn laptop_test() {
        assert!(activate_device(
            "CoreAudio",
            "Built-in Microphone",
            "Built-in Output",
            1,
            2,
            48000,
            512,
            {
                let start = AtomicI64::new(0);
                let mic_gain = 0.0; // 2.5 --- you risk feedback and destroying your speakers. be sure you know what you are doing. but it's a cool test :)

                Box::new(move |inputs, outputs, num_samples| {
                    let mut f_start = start.load(Ordering::Relaxed) as f64;

                    for i in 0..num_samples {
                        let mic = inputs[0][i] * mic_gain;
                        outputs[0][i] = f_start.sin() as f32 * 0.5 + mic;
                        outputs[1][i] = f_start.sin() as f32 * 0.5 + mic;
                        f_start += 0.5;
                    }

                    start.store(f_start as i64, Ordering::Relaxed);
                })
            }
        )
        .is_ok());

        std::thread::sleep(Duration::from_secs(5));

        stop_devices();
    }
}
