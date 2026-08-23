pub(crate) fn make_icon_rgba() -> (Vec<u8>, u32, u32) {
    const WIDTH: u32 = 32;
    const HEIGHT: u32 = 32;
    let mut rgba = vec![0_u8; (WIDTH * HEIGHT * 4) as usize];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = ((y * WIDTH + x) * 4) as usize;
            let inside = (3..29).contains(&x) && (3..29).contains(&y);
            let grid_line = inside && (x == 12 || x == 20 || y == 12 || y == 20);

            let color = if grid_line {
                [245, 248, 255, 255]
            } else if inside {
                [65, 105, 225, 255]
            } else {
                [0, 0, 0, 0]
            };

            rgba[index..index + 4].copy_from_slice(&color);
        }
    }

    (rgba, WIDTH, HEIGHT)
}
