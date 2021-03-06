use std::collections::HashSet;

fn run_strange_program() -> u64 {
    let reg0 = 0;

    let mut numbers_found = HashSet::new();
    let mut prev_reg_3 = 0;

    let mut reg3: u64 = 0;
    'outer: loop {
        let mut reg2 = reg3 | 65536;
        reg3 = 1397714;

        loop {
            let mut reg5 = reg2 & 0xFF;
            reg3 += reg5;
            reg3 &= 16777215;
            reg3 *= 65899;
            reg3 &= 16777215;
            if 256 > reg2 {
                // println!("reg3 {:?}", reg3);
                if reg3 == reg0 {
                    break 'outer;
                } else {
                    if !numbers_found.insert(reg3) {
                        return prev_reg_3;
                    }
                    prev_reg_3 = reg3;
                    if numbers_found.len() % 1000 == 0 {
                        println!("numbers_found {}", numbers_found.len());
                    }
                    continue 'outer;
                }
            }

            reg5 = 0;
            loop {
                let mut reg1 = reg5 + 1;
                reg1 *= 256;
                if reg1 > reg2 {
                    break;
                }
                reg5 += 1;
            }
            reg2 = reg5;
        }
    }

    panic!("WTF?");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_21_weird_program() {
        assert_eq!(12333799, run_strange_program());
    }
}

fn main() {
    println!("{}", run_strange_program());
}
