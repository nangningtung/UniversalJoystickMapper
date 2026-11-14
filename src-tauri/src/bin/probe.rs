#[repr(C)]
struct JoyCapsW {
    w_mid: u16,
    w_pid: u16,
    sz_pname: [u16; 32],
    w_xmin: u32,
    w_xmax: u32,
    w_ymin: u32,
    w_ymax: u32,
    w_zmin: u32,
    w_zmax: u32,
    w_num_buttons: u32,
    w_period_min: u32,
    w_period_max: u32,
    w_rmin: u32,
    w_rmax: u32,
    w_umin: u32,
    w_umax: u32,
    w_vmin: u32,
    w_vmax: u32,
    w_caps: u32,
    w_max_axes: u32,
    w_num_axes: u32,
    w_max_buttons: u32,
    sz_reg_key: [u16; 32],
    sz_oem_vx_d: [u16; 260],
}

#[repr(C)]
struct JoyInfoEx {
    dw_size: u32,
    dw_flags: u32,
    dw_xpos: u32,
    dw_ypos: u32,
    dw_zpos: u32,
    dw_rpos: u32,
    dw_upos: u32,
    dw_vpos: u32,
    dw_buttons: u32,
    dw_button_number: u32,
    dw_pov: u32,
    dw_reserved1: u32,
    dw_reserved2: u32,
}

#[link(name = "winmm")]
extern "system" {
    fn joyGetNumDevs() -> u32;
    fn joyGetDevCapsW(u_joy_id: usize, pjc: *mut JoyCapsW, cbjc: u32) -> u32;
    fn joyGetPosEx(u_joy_id: usize, pji: *mut JoyInfoEx) -> u32;
}

fn main() {
    println!("Polling joystick 0 for 3 seconds... PRESS BUTTONS ON M-TECH STICK!");
    let start = std::time::Instant::now();
    let mut last_btn = 0xFFFFFFFFu32;
    while start.elapsed() < std::time::Duration::from_secs(3) {
        let mut info: JoyInfoEx = unsafe { std::mem::zeroed() };
        info.dw_size = std::mem::size_of::<JoyInfoEx>() as u32;
        info.dw_flags = 0xFF; // JOY_RETURNALL
        let res = unsafe { joyGetPosEx(0, &mut info) };
        if res == 0 && info.dw_buttons != last_btn {
            last_btn = info.dw_buttons;
            println!("Buttons changed: 0x{:04X} | X={} Y={} Z={} R={} POV={}",
                info.dw_buttons, info.dw_xpos, info.dw_ypos, info.dw_zpos, info.dw_rpos, info.dw_pov);
            for b in 0..16 {
                if (info.dw_buttons & (1 << b)) != 0 {
                    println!("  -> BUTTON {} IS PRESSED!", b);
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
}

