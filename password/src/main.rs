#[derive(Debug)]
struct Password {
    decrease: bool,
    repeat: bool,
    invalid_repeat: bool,
    repeat_len: u32,
    len: u32,
    value: u32,
}

impl Password {
    fn new(val: u32, len: u32) -> Self {
        Password {
            decrease: false,
            repeat: false,
            invalid_repeat: false,
            repeat_len: 1,
            len: len,
            value: val,
        }
    }

    fn valid(&mut self) -> bool {
        let mut div = 10_u32.pow(self.len - 1);
        let mut previous = self.value/div;

        for pos in 2..(self.len + 1) {
            self.value = self.value - div * previous;
            div = div/10;

            let digit = self.value/div;

            self.check_repeat(digit, previous, pos == self.len);

            if self.decrease {
                break;
            } else {
                self.decrease = digit < previous;
            }

            //dbg!(&self);
            //dbg!(previous);
            //dbg!(digit);

            previous = digit;
        }

        !self.decrease && self.repeat && !self.invalid_repeat
    }

    fn check_repeat(&mut self, digit: u32, previous: u32, at_end: bool) {
        if digit == previous {
            self.repeat = true;
            self.repeat_len = self.repeat_len + 1;
            if at_end && (self.repeat_len % 2 == 1) { self.invalid_repeat = true; }
        } else {
            if (self.repeat_len != 1) && (self.repeat_len % 2 == 1) { self.invalid_repeat = true; }
            self.repeat_len = 1;
        }
    }
}

fn main() {
    println!("Potential passwords: {}", check(123257, 647015));
}

fn check(start: u32, stop: u32) -> u32 {
    let mut potential = 0;

    for pass in start..stop {
        let mut password = Password::new(pass, 6);

        if password.valid() { potential = potential + 1; }
    }

    potential
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_valid() {
        let mut password = Password::new(123445, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn invalid_decrease() {
        let mut password = Password::new(122144, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn invalid_no_repeat() {
        let mut password = Password::new(123456, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn invalid_triple() {
        let mut password = Password::new(122234, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn valid_quad() {
        let mut password = Password::new(111123, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn invalid_triple_with_valid_repeat() {
        let mut password = Password::new(222334, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn invalid_triple_at_end() {
        let mut password = Password::new(112333, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn invalid_triple_end_with_valid_double() {
        let mut password = Password::new(122444, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn fiver() {
        let mut password = Password::new(122222, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn double_triple() {
        let mut password = Password::new(111222, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn triple_double() {
        let mut password = Password::new(112233, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn triple_then_double() {
        let mut password = Password::new(111233, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn four_ender() {
        let mut password = Password::new(123333, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn five_ender() {
        let mut password = Password::new(122222, 6);

        assert_eq!(false, password.valid());
    }

    #[test]
    fn four_starter() {
        let mut password = Password::new(111123, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn sixer() {
        let mut password = Password::new(111111, 6);

        assert_eq!(true, password.valid());
    }

    #[test]
    fn five_starter() {
        let mut password = Password::new(444445, 6);

        assert_eq!(false, password.valid());
    }
}