use crate::errors::PiShockError;
use crate::PiShocker;
use log::{debug, info};
use std::time::Duration;
use textplots::{Chart, Plot, Shape};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShockPoint {
    duration: Duration,
    intensity: u32,
}

impl ShockPoint {
    #[must_use]
    pub fn new(duration: Duration, intensity: u32) -> Self {
        Self {
            duration,
            intensity,
        }
    }
}

// The distance in ms between each point in the interpolated curve
// Setting this too low *will* cause DeviceBusy errors
static INTERPOLATION_RESOLUTION: u32 = 500;
static INTERPOLATION_SLEEP_TIME: u32 = 100;

static INTERPOLATION_DEFAULT_START_Y: u32 = 1;

impl PiShocker {
    pub async fn shock_curve(&self, points: Vec<ShockPoint>) -> Result<(), PiShockError> {
        // Verify that all ShockPoints don't exceed duration or intensity limits
        for point in &points {
            if let Some(error) = self.max_intensity_error_triggered(point.intensity) {
                return Err(error);
            }

            if let Some(error) = self.max_duration_error_triggered(point.duration) {
                return Err(error);
            }
        }

        let total_length: Duration = points.iter().map(|point| point.duration).sum();
        debug!("Total length of raw curve: {:#?}", total_length);

        // Calculate an interpolated curve
        let mut interpolated_curve: Vec<ShockPoint> = Vec::new();

        // Generate a Vec with shock points that are interpolated between the given points
        for point in points {
            let mut time = Duration::default();
            while time < point.duration {
                let intensity = linear_interpolation(
                    interpolated_curve
                        .last()
                        .map_or(INTERPOLATION_DEFAULT_START_Y, |point| point.intensity),
                    point.intensity,
                    time.as_millis() as u32,
                    point.duration.as_millis() as u32,
                );

                interpolated_curve.push(ShockPoint::new(
                    Duration::from_millis(u64::from(INTERPOLATION_RESOLUTION)),
                    intensity,
                ));
                time += Duration::from_millis(u64::from(INTERPOLATION_RESOLUTION));
            }
        }

        {
            let shape = Shape::Continuous(Box::new(|x| {
                interpolated_curve
                    .clone()
                    .get(x as usize)
                    .map_or(0.0, |point| point.intensity as f32)
            }));

            let mut chart = Chart::new(180, 60, 0 as f32, interpolated_curve.len() as f32);
            let chart_line = chart.lineplot(&shape);

            info!(
                "Shock step graph - 1 step = {}ms\n{}",
                INTERPOLATION_RESOLUTION,
                chart_line.to_string()
            );
        }

        #[cfg(debug_assertions)]
        debug!("Interpolated curve: {:#?}", interpolated_curve);

        debug!(
            "Total length of interpolated curve: {:#?}",
            interpolated_curve
                .iter()
                .map(|point| point.duration)
                .sum::<Duration>()
        );

        for point in interpolated_curve {
            debug!(
                "Sending shock at intensity {} for duration {:#?}",
                point.intensity, point.duration
            );
            self.shock(point.intensity, point.duration).await?;
            tokio::time::sleep(Duration::from_millis(u64::from(INTERPOLATION_SLEEP_TIME))).await;
        }
        debug!("Finished sending shock curve");

        Ok(())
    }
}

fn linear_interpolation(start: u32, end: u32, time: u32, duration: u32) -> u32 {
    // Convert to signed to allow for negative values that will
    // be present during downward trends
    let start = start as i32;
    let end = end as i32;
    let time = time as i32;
    let duration = duration as i32;
    let result = start + (end - start) * time / duration;

    debug!(
        "Linear interpolation: start: {}, end: {}, time: {}, duration: {}",
        start, end, time, duration
    );
    debug!("Linear interpolation: {}", result);

    // The absolute intensity must always be a positive value
    result.unsigned_abs()
}
