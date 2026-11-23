use super::EnergySettings;
use crate::model::Station;
use chrono::{DateTime, Utc};

/// Kinds of interaction that add energy to a station.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touch {
    Select,
    Edit,
    Connect,
    Drag,
    Rekindle,
}

/// Lazy exponential decay:
///
/// `E(now) = energy_at_touch * 0.5 ^ ((now − last_touched) / half_life)`
///
/// A `last_touched` in the future (clock skew, a file from another machine) is
/// treated as "just now" so energy can never exceed what was stored.
pub fn decayed(
    energy_at_touch: f64,
    last_touched: DateTime<Utc>,
    now: DateTime<Utc>,
    half_life_secs: f64,
) -> f64 {
    let elapsed = ((now - last_touched).num_milliseconds() as f64 / 1000.0).max(0.0);
    let factor = if half_life_secs > 0.0 {
        0.5f64.powf(elapsed / half_life_secs)
    } else {
        1.0
    };
    (energy_at_touch * factor).clamp(0.0, 1.0)
}

/// A bump: `E = min(1, E(now) + amount)`, stored with `last_touched = now`.
pub fn bump(station: &mut Station, touch: Touch, now: DateTime<Utc>, settings: &EnergySettings) {
    let current = station.energy(now, settings);
    station.energy_at_touch = (current + settings.amount(touch)).min(1.0);
    station.last_touched = now;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StationId;
    use approx::assert_relative_eq;
    use chrono::Duration;
    use glam::Vec2;

    const T0: DateTime<Utc> = DateTime::UNIX_EPOCH;

    #[test]
    fn energy_halves_after_one_half_life() {
        let s = EnergySettings::default();
        let later = T0 + Duration::days(3);
        assert_relative_eq!(
            decayed(0.8, T0, later, s.half_life_secs),
            0.4,
            epsilon = 1e-9
        );
    }

    #[test]
    fn energy_is_unchanged_at_touch_time() {
        assert_relative_eq!(decayed(0.7, T0, T0, 100.0), 0.7);
    }

    #[test]
    fn future_touch_does_not_amplify() {
        let earlier = T0 - Duration::days(1);
        assert_relative_eq!(decayed(0.5, T0, earlier, 100.0), 0.5);
    }

    #[test]
    fn bump_adds_to_decayed_energy_and_caps_at_one() {
        let s = EnergySettings::default();
        let mut station = Station::new(StationId(1), "x", Vec2::ZERO, T0, &s);
        let later = T0 + Duration::days(3);
        bump(&mut station, Touch::Edit, later, &s);
        assert_relative_eq!(station.energy_at_touch, 0.3 + 0.2, epsilon = 1e-9);
        assert_eq!(station.last_touched, later);
        for _ in 0..10 {
            bump(&mut station, Touch::Rekindle, later, &s);
        }
        assert_relative_eq!(station.energy_at_touch, 1.0);
    }
}
