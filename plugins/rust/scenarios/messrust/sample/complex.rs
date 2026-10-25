pub fn deeply_complex_function(
    a: i32,
    b: i32,
    c: i32,
    d: i32,
    e: i32,
    f: i32,
) -> i32 {
    let mut sum = a + b + c + d + e + f;
    if a > 0 {
        if b > 0 {
            if c > 0 {
                sum += 1;
            } else {
                sum -= 1;
            }
        } else if d > 0 {
            sum += 2;
        }
    } else if b < 0 {
        match c {
            1 => sum += 10,
            2 => sum += 20,
            3 => sum += 30,
            4 => sum += 40,
            _ => sum += 50,
        }
    }

    if d < 0 {
        for i in 0..10 {
            sum += i;
        }
    } else {
        sum *= 2;
    }

    sum
}
