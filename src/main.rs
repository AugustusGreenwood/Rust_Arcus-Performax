use rusb::{
    DeviceHandle, devices, GlobalContext, DeviceDescriptor
};

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);


// Begin functions

fn _write_to_control(handle: &mut DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(_n) => (),
        Err(e) => {
            println!("%%%% Couldn't write to control buffer.\n%%%% Non-critical error: \"{}\"\n", e);
            return Err(e);
        },
    };
    Ok(())
}

pub fn open(vid: u16, pid: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let mut handle__: Option<DeviceHandle<GlobalContext>> = None;
    for device in devices()?.iter() {
        let device_desc: DeviceDescriptor = device.device_descriptor()?;

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            handle__ = Some(device.open()?);
            break;
        }
    }

    let mut handle: DeviceHandle<GlobalContext> = match handle__ {
        Some(handle) => handle,
        
        None => {
            println!("Couldn't find device, make sure it is on and plugged in.\n");
            return Err(rusb::Error::NotFound);
        },
    };

    match handle.claim_interface(0) {
        Ok(()) => (),
        
        Err(e) => {
            println!("%%%% Couldn't claim interface!\n%%%% Critical error\n%%%% Exiting\n");
            return Err(e);
        },
    };

    _write_to_control(&mut handle, 2)?;
    
    match handle.read_bulk(0x82, &mut [0; 4096], TIMEOUT) {
        Ok(_n) => (),
        
        Err(e) => println!("%%%% Couldn't clear read buffer.\n%%%% Non-critical error: \"{}\"\n", e),
    };
    Ok(handle)
}

fn _extract_response_from_raw_output(output: &mut Vec<u8>) -> String {
    let mut response: Vec<u8> = Vec::new();
    for &i in output.iter() {
        if i == 0 {break;}
        response.push(i);
    }
    std::str::from_utf8(&response).unwrap().to_string()
}

fn _write_to_bulk(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written: usize = match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => n,
        
        Err(e) => {
            println!("Couldn't bulk write with error: {}\nExiting to be safe\n", e);
            return Err(e);
        },
    };

    if bytes_written != command.len() {
        println!("Incorrent number of bytes written. Command was likely not sent properly. {} bytes written when {} were expected
                with command {:#?}.", bytes_written, command.len(), command);
    }

    Ok(())
}

fn _read_from_bulk(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let output: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    let response: String = match handle.read_bulk(0x82, output, TIMEOUT) {
        Ok(_n) => _extract_response_from_raw_output(output),
        Err(e) => {
            println!("Couldn't bulk read with error{:#?}\n", e);
            return Err(e);
        },
    };  
    Ok(response)
}

pub fn send_command_get_response(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<String> {
    _write_to_bulk(handle, command)?;

    let response: String = _read_from_bulk(handle)?;

    Ok(response)
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    _write_to_control(handle, 4)?;
    
    match handle.release_interface(0) {
        Ok(()) => (),
        Err(e) => {
            println!("Couldn't release interface! Possibly bad, you might need to restart the device\n");
            return Err(e);
        },
    };
    Ok(())
}

fn _check_for_valid_response(response: &String, error_log: &str) -> rusb::Result<()> {
    if response.starts_with('?') {
        println!("Response {} is invalid.\n{}", response, error_log);
        return Err(rusb::Error::Io);
    }
    Ok(())
}

// Begin functions

fn main() {
    println!("Hello World!");
}
