use crate::error::GoliathVehicleResult;
use tinyvec::ArrayVec;

// Specialized NEON implementation for ARM64 for image packing, technically saves about 75% of the time
// But the time is already like 12Microseconds, so the gain is not that significant
// Still, was fun to write, and the performance gain if writing to RAM might be more significant
#[cfg(target_arch = "aarch64")]
#[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
unsafe fn image_to_screen_space(
    image_raw: &[u8],
    screen_width: usize,
    page_count: usize,
) -> ArrayVec<[u8; 1024]> {
    use std::{arch, hint};

    let mut buffer = ArrayVec::from_array_len([0; 1024], page_count * screen_width);

    // 8 Steps in the outer loop, each step is 16 bytes wide
    (0..screen_width).step_by(16).for_each(|section_start_col| {
        // Max 8 pages, each page is 8 rows high
        (0..page_count).for_each(|page_idx| unsafe {
            let page_start_row = page_idx * 8;

            // Each page has 8 rows
            let acc_lane = (0..8).fold(arch::aarch64::vdupq_n_u8(0), |acc_lane, row_in_page| {
                let row_idx = page_start_row + row_in_page;
                // Load 128 bits (16 bytes) starting at this section and row
                let row_lane = arch::aarch64::vld1q_u8(
                    image_raw[row_idx * screen_width + section_start_col..].as_ptr(),
                );

                // We only care about the MSBs of each byte in the row(I.e., greater than 127 or not)
                let lane_with_msbs = arch::aarch64::vshrq_n_u8::<7>(row_lane);

                // Shift the MSBs to the correct position in the final byte
                // Each row only takes one bit in the final byte, based on its position in the page
                let shifted_lane = match row_in_page {
                    0 => arch::aarch64::vshlq_n_u8::<0>(lane_with_msbs),
                    1 => arch::aarch64::vshlq_n_u8::<1>(lane_with_msbs),
                    2 => arch::aarch64::vshlq_n_u8::<2>(lane_with_msbs),
                    3 => arch::aarch64::vshlq_n_u8::<3>(lane_with_msbs),
                    4 => arch::aarch64::vshlq_n_u8::<4>(lane_with_msbs),
                    5 => arch::aarch64::vshlq_n_u8::<5>(lane_with_msbs),
                    6 => arch::aarch64::vshlq_n_u8::<6>(lane_with_msbs),
                    7 => arch::aarch64::vshlq_n_u8::<7>(lane_with_msbs),
                    _ => hint::unreachable_unchecked(),
                };

                // Do a bitwise OR of the shifted lane with the accumulated lane
                arch::aarch64::vorrq_u8(acc_lane, shifted_lane)
            });

            // Store the accumulated lane into the buffer at the correct position
            arch::aarch64::vst1q_u8(
                &mut buffer[page_idx * screen_width + section_start_col],
                acc_lane,
            );
        });
    });

    buffer
}

#[cfg(not(target_arch = "aarch64"))]
#[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
fn image_to_screen_space(
    image_raw: &[u8],
    screen_width: usize,
    page_count: usize,
) -> ArrayVec<[u8; 1024]> {
    let mut buffer = ArrayVec::from_array_len([0; 1024], page_count * screen_width);
    (0..screen_width).for_each(|col_idx| {
        (0..page_count).for_each(|page_idx| {
            let page_start_idx = page_idx * screen_width;
            let col_byte = (0..8).fold(0, |acc, row_idx_in_page| {
                let row_idx = page_idx * 8 + row_idx_in_page;
                let normalized_byte = image_raw[row_idx * screen_width + col_idx] >> 7; // Normalize to 0 or 1
                let page_bit = row_idx % 8;
                let result_byte = normalized_byte << page_bit;
                acc | result_byte
            });

            buffer[page_start_idx + col_idx] = col_byte;
        })
    });

    buffer
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
pub(crate) fn resize_image(
    image: image::GrayImage,
    screen_width: u32,
    screen_height: u32,
) -> image::GrayImage {
    if image.width() == screen_width && image.height() == screen_height {
        image
    } else {
        image::imageops::resize(
            &image,
            screen_width,
            screen_height,
            image::imageops::FilterType::Nearest,
        )
    }
}

pub(crate) fn convert_image_to_screen_space(
    image: image::GrayImage,
    screen_width: u32,
    screen_height: u32,
) -> GoliathVehicleResult<ArrayVec<[u8; 1024]>> {
    let page_count = screen_height / 8;

    #[allow(unused_unsafe)]
    Ok(unsafe { image_to_screen_space(&image, screen_width as usize, page_count as usize) })
}

#[cfg_attr(feature = "trace", tracing::instrument(level = "info", skip_all))]
pub(crate) fn load_goliath_logo() -> GoliathVehicleResult<image::GrayImage> {
    image::load_from_memory_with_format(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources",
            "/Goliath.png"
        )),
        image::ImageFormat::Png,
    )
    .map(|img| img.into_luma8())
    .map_err(Into::into)
}
