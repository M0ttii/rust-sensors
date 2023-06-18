use windows::{core::*, Win32::System::Com::*, Win32::System::Ole::*, Win32::System::Wmi::*};
use std::{thread, time};
use win_ring0::WinRing0;


pub fn main() {
    let mut r0: Box<WinRing0> = Box::from(WinRing0::new());


    println!("Installing ring0 driver");
    match r0.install() {
        Ok(()) => { println!("Driver installed"); }
        Err(err) => { println!("Error: {}", err); }
    }

    println!("Opening ring0 driver");
    match r0.open() {
        Ok(()) => { println!("Driver opened"); }
        Err(err) => { println!("Error: {}", err); }
    }

    let max_temp = 0x1a2;
    let msr = 0x19c;
    let msr2 = 0x1b1;
    let out = r0.readMsr(msr2).unwrap();
    let max_temp_read = r0.readMsr(max_temp).unwrap();
    println!("RAW: {}", out);

    let _edx = ((max_temp_read >> 32) & 0xffffffff) as u32;
    let eax = (max_temp_read & 0xffffffff) as u32;
    let max_temp_value = (eax >> 16) & 0x7f;
    println!("MAX: {}", max_temp_value);

    let delay = time::Duration::from_secs(2);

    loop {
        let _edx = ((out >> 32) & 0xffffffff) as u32;
        let eax = (out & 0xffffffff) as u32;
        let tj_max = (eax >> 16) & 0x3f;
        let real = (max_temp_value - tj_max);
        println!("Package Temperature: {}", real);
        thread::sleep(delay);
    }

    println!("Closing ring0 driver");
    match r0.close() {
        Ok(()) => { println!("Driver closed"); }
        Err(err) => { println!("Error: {}", err); }
    }

    println!("Uninstall ring0 driver");
    match r0.uninstall() {
        Ok(()) => { println!("Driver uninstalled"); }
        Err(err) => { println!("Error: {}", err); }
    }
}

pub fn deinstall(mut r0: Box<WinRing0>){
    ctrlc::set_handler(move || {
        println!("Closing ring0 driver");
        match r0.close() {
            Ok(()) => { println!("Driver closed"); }
            Err(err) => { println!("Error: {}", err); }
        }

        println!("Uninstall ring0 driver");
        match r0.uninstall() {
            Ok(()) => { println!("Driver uninstalled"); }
            Err(err) => { println!("Error: {}", err); }
        }
    })
        .expect("Error setting Ctrl-C handler");
}

fn wmi_get_value() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        )?;

        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;

        let server =
            locator.ConnectServer(&BSTR::from("root\\cimv2"), None, None, None, 0, None, None)?;

        let query = server.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("select SystemName, CurrentReading from CIM_TemperatureSensor"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )?;

        loop {
            let mut row = [None; 1];
            let mut returned = 0;
            query.Next(WBEM_INFINITE, &mut row, &mut returned).ok()?;

            if let Some(row) = &row[0] {
                let mut value = Default::default();
                let mut value1 = Default::default();
                row.Get(w!("SystemName"), 0, &mut value, None, None)?;
                row.Get(w!("CurrentReading"), 0, &mut value1, None, None)?;
                println!(
                    "{}",
                    VarFormat(
                        &value,
                        None,
                        VARFORMAT_FIRST_DAY_SYSTEMDEFAULT,
                        VARFORMAT_FIRST_WEEK_SYSTEMDEFAULT,
                        0
                    )?
                );

                // TODO: workaround for https://github.com/microsoft/windows-rs/issues/539
                VariantClear(&mut value)?;
            } else {
                break;
            }
        }

        Ok(())
    }
}
