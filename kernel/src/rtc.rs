use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "Jan",
        }
    }

    pub fn weekday_name(&self) -> &'static str {
        let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year as i32;
        let m = (self.month.clamp(1, 12)) as usize;
        if m < 3 {
            y -= 1;
        }
        let dow = ((y + y / 4 - y / 100 + y / 400 + t[m - 1] + self.day as i32) % 7).unsigned_abs() as usize;
        match dow {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            _ => "Sat",
        }
    }

    pub fn format_time(&self) -> alloc::string::String {
        alloc::format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }

    pub fn format(&self) -> alloc::string::String {
        alloc::format!(
            "{} {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            self.weekday_name(),
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )
    }
}

fn read_cmos(reg: u8) -> u8 {
    let mut addr_port: Port<u8> = Port::new(0x70);
    let mut data_port: Port<u8> = Port::new(0x71);
    unsafe {
        addr_port.write(reg);
        data_port.read()
    }
}

fn is_updating() -> bool {
    let mut addr_port: Port<u8> = Port::new(0x70);
    let mut data_port: Port<u8> = Port::new(0x71);
    unsafe {
        addr_port.write(0x0A);
        (data_port.read() & 0x80) != 0
    }
}

fn bcd_to_bin(val: u8) -> u8 {
    (val & 0x0F) + ((val / 16) * 10)
}

pub fn read_rtc() -> DateTime {
    // Wait until RTC is not in the middle of an update
    for _ in 0..10_000 {
        if !is_updating() {
            break;
        }
    }

    let mut second = read_cmos(0x00);
    let mut minute = read_cmos(0x02);
    let mut hour = read_cmos(0x04);
    let mut day = read_cmos(0x07);
    let mut month = read_cmos(0x08);
    let mut year = read_cmos(0x09);
    let reg_b = read_cmos(0x0B);

    // Convert BCD to binary if bit 2 of Register B is not set
    if (reg_b & 0x04) == 0 {
        second = bcd_to_bin(second);
        minute = bcd_to_bin(minute);
        hour = bcd_to_bin(hour & 0x7F) | (hour & 0x80);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
    }

    // Convert 12-hour to 24-hour if bit 1 of Register B is not set and PM bit is set
    if (reg_b & 0x02) == 0 && (hour & 0x80) != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    let full_year = 2000 + (year as u16);

    DateTime {
        year: full_year,
        month,
        day,
        hour,
        minute,
        second,
    }
}
