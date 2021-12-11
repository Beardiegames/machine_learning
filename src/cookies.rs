use std::{
    ops::{Add, Sub, AddAssign, SubAssign}, 
    cmp::Ordering
};


#[derive(Default, Copy, Clone)]
pub struct Cookies (pub u8);

impl Cookies {
    pub fn half(self) -> Cookies {
        if self.0 > 1 { 
            Cookies(self.0 / 2)
        } else {
            Cookies(0)
        }
    }
}

impl PartialEq for Cookies {
    fn eq(&self, other: &Cookies) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Cookies) -> bool {
        self.0 != other.0
    }
}

impl PartialOrd for Cookies {
    fn partial_cmp(&self, other: &Cookies) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }

    fn lt(&self, other: &Cookies) -> bool { self.0 < other.0 }
    fn le(&self, other: &Cookies) -> bool { self.0 <= other.0 }
    fn gt(&self, other: &Cookies) -> bool { self.0 > other.0 }
    fn ge(&self, other: &Cookies) -> bool { self.0 >= other.0 }
}

impl Add for Cookies {
    type Output = Cookies;
    fn add(self, other: Cookies) -> Cookies {
        if other.0 < 255 - self.0 {
            Cookies(self.0 + other.0)
        } else {
            Cookies(255)
        }
    }
}

impl AddAssign for Cookies {
    fn add_assign(&mut self, other: Cookies) {
        if other.0 < 255 - self.0 {
            self.0 = self.0 + other.0
        } else {
            self.0 = 255
        }
    }
}


impl Sub for Cookies {
    type Output = Cookies;
    fn sub(self, other: Cookies) -> Cookies {
        if other.0 < self.0 {
            Cookies(self.0 - other.0)
        } else {
            Cookies(0)
        }
    }
}

impl SubAssign for Cookies {
    fn sub_assign(&mut self, other: Cookies) {
        if other.0 < self.0 {
            self.0 = self.0 - other.0;
        } else {
            self.0 = 0;
        }
    }
}