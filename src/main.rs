use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::FALSE;
use winapi::shared::ntdef::NULL;
use winapi::um::processthreadsapi;
use winapi::um::tlhelp32;
use winapi::um::winnt;

use reqwest;

const EXECUTABLES: [&str; 1] = ["League of Legends.exe"];
const SECOND_INTERVAL: u64 = 60;

fn main() {
    let monitor = Monitor::new();
    monitor.start();
}

struct Monitor {
    client: reqwest::Client,
}
impl Monitor {
    fn new() -> Self {
        Monitor {
            client: reqwest::Client::new(),
        }
    }

    fn start(&self) -> ! {
        let dur = std::time::Duration::from_secs(SECOND_INTERVAL);

        loop {
            let start = std::time::Instant::now();

            // check to see if a process is running that we will pause for
            let send_stop = unsafe { get_process_list() };

            match send_stop {
                true => {
                    // send the stop request
                    let send = self
                        .client
                        .post("http://192.168.86.139:9932/pause_torrent")
                        .send();

                    // check to see the response of the
                    match send {
                        Ok(_) => println! {"sent request to server to pause torrents"},
                        Err(e) => {
                            println! {"server is not running. One of a few conditions is possible:\n\t1. The server is not running (not VPN'd + not torrenting, or server should actually be up) \n\t2.Your sys-monitor version is out of date and your requests are no longer being considered\n\nError Information: {:?}", e}
                        }
                    }
                }
                false => (),
            }

            // sleep until the next time we update
            let new_time = dur - start.elapsed();
            std::thread::sleep(new_time);
        }
    }
}

/// Fetches all top-level processes that are running
unsafe fn get_process_list() -> bool {
    let h_process_snap = tlhelp32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPPROCESS, 0);
    let mut pe32: tlhelp32::PROCESSENTRY32 = tlhelp32::PROCESSENTRY32::default();
    pe32.dwSize = std::mem::size_of::<tlhelp32::PROCESSENTRY32>() as u32;

    let pe32_ptr: *mut tlhelp32::PROCESSENTRY32 = &mut pe32 as *mut tlhelp32::PROCESSENTRY32;
    let first_process = tlhelp32::Process32First(h_process_snap, pe32_ptr);

    dbg! {first_process};
    if first_process != 1 {
        // supposed to CloseHandle here but the function
        // does not exist
    }

    loop {
        let process = tlhelp32::Process32Next(h_process_snap, pe32_ptr);

        if process != 1 {
            // dbg!{"exiting loop"};
            break;
        }

        let open_process =
            processthreadsapi::OpenProcess(winnt::PROCESS_ALL_ACCESS, FALSE, pe32.th32ProcessID);

        if open_process == NULL {
            // println!{"NULL open_process"}
            continue;
        }
        // dbg!{open_process};

        let process_class = processthreadsapi::GetPriorityClass(open_process);
        if process_class == 0 {
            // println!{"zero process class"}
        }
        match list_process_modules(pe32.th32ProcessID) {
            true => return true,
            false => (),
        }
    }
    false
}

/// Fetches all modules running under a top-level process
unsafe fn list_process_modules(process_id: DWORD) -> bool {
    let h_module_snap = tlhelp32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPMODULE, process_id);

    if h_module_snap == winapi::um::handleapi::INVALID_HANDLE_VALUE {
        // println!{"invalid module value"}
    }

    let mut me32 = tlhelp32::MODULEENTRY32::default();
    me32.dwSize = std::mem::size_of::<tlhelp32::MODULEENTRY32>() as u32;

    let me32_ptr = &mut me32 as *mut _;
    let first_module = tlhelp32::Module32First(h_module_snap, me32_ptr);

    if first_module != 1 {
        // println!{"first module was not 1"}
        return false;
    }

    loop {
        let next_module = tlhelp32::Module32Next(h_module_snap, me32_ptr);

        // if windows says the module contains no information
        if next_module != 1 {
            // println!{"next module was not 1, breaking"}
            break;
        }

        // convert 200-lenth array of i8 to Vec<u8>
        let name = me32
            .szExePath
            .iter()
            .filter(|x| **x != 0)
            .map(|x| *x as u8)
            .collect::<Vec<_>>();

        // convert the processs name
        // TODO: Compare bytes instead of converting
        let name = String::from_utf8_lossy(&name);

        // check the module to see if it matches any of the processes being monitored
        for executable in &EXECUTABLES {
            if name.contains(executable) {
                // println! {"found match for {} : {}", executable, name}
                return true;
            }
        }
    }

    false
}
