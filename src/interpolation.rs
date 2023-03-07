use std::time::Duration;
use log::debug;
use crate::errors::PiShockError;
use crate::PiShocker;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ShockPoint {
    duration: Duration,
    intensity: u32,
}

impl ShockPoint {
    pub fn new(duration: Duration, intensity: u32) -> Self {
        Self {
            duration,
            intensity,
        }
    }
}


// The distance in ms between each point in the interpolated curve
// Setting this too low *will* cause DeviceBusy errors
static INTERPOLATION_RESOLUTION: u32 = 1000;

impl PiShocker {

    pub async fn shock_curve(&self, points: Vec<ShockPoint>) -> Result<(), PiShockError> {
        // Verify that all ShockPoints don't exceed duration or intensity limits
        for point in &points {
            if self.max_intensity_error_triggered(point.intensity).is_some() {
                return Err(self.max_intensity_error_triggered(point.intensity).unwrap());
            }

            if self.max_duration_error_triggered(point.duration).is_some() {
                return Err(self.max_duration_error_triggered(point.duration).unwrap());
            }
        }

        let total_length : Duration = points.iter().map(|point| point.duration).sum();

        // Calculate an interpolated curve
        let mut interpolated_curve: Vec<ShockPoint> = Vec::new();

        let current_time = Duration::default();

        // Generate a Vec with shock points that are interpolated between the given points
        for point in points {
            let mut time = Duration::default();
            while time < point.duration {
                let intensity = linear_interpolation(
                    interpolated_curve.last().map(|point| point.intensity).unwrap_or(1),
                    point.intensity,
                    time.as_millis() as u32,
                    point.duration.as_millis() as u32,
                );
                interpolated_curve.push(ShockPoint::new(
                    Duration::from_millis(INTERPOLATION_RESOLUTION as u64),
                    intensity,
                ));
                time += Duration::from_millis(INTERPOLATION_RESOLUTION as u64);
            }
        }

        #[cfg(debug_assertions)]
        debug!("Interpolated curve: {:#?}", interpolated_curve);

        for point in interpolated_curve {
            debug!("Sending shock at intensity {} for duration {:#?}", point.intensity, point.duration);
            self.shock(point.intensity, point.duration).await?;
            tokio::time::sleep(Duration::from_millis(INTERPOLATION_RESOLUTION as u64)).await;
        }
        debug!("Finished sending shock curve");

        Ok(())
    }
}

fn linear_interpolation(start: u32, end: u32, time: u32, duration: u32) -> u32 {
    // Convert to signed to allow for negative values
    let start = start as i32;
    let end = end as i32;
    let time = time as i32;
    let duration = duration as i32;
    let result = start + (end - start) * time / duration;

    debug!("Linear interpolation: start: {}, end: {}, time: {}, duration: {}", start, end, time, duration);
    debug!("Linear interpolation: {}", result);

    result as u32
}
