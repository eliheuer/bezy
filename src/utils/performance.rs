//! Performance monitoring and profiling tools
//! 
//! This module provides tools to measure and track performance improvements
//! during the refactoring process.

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Resource to track system execution times
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    /// System execution times over the last N frames
    pub system_times: HashMap<String, Vec<Duration>>,
    /// Frame times over the last N frames
    pub frame_times: Vec<Duration>,
    /// Memory usage snapshots
    pub memory_snapshots: Vec<MemorySnapshot>,
    /// Maximum number of samples to keep
    pub max_samples: usize,
    /// Last frame start time
    pub last_frame_start: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub entities_count: usize,
    pub components_count: usize,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            max_samples: 120, // Keep 2 seconds of data at 60 FPS
            ..Default::default()
        }
    }
    
    /// Record a system execution time
    pub fn record_system_time(&mut self, system_name: String, duration: Duration) {
        let times = self.system_times.entry(system_name).or_insert_with(Vec::new);
        times.push(duration);
        
        // Keep only the last N samples
        if times.len() > self.max_samples {
            times.remove(0);
        }
    }
    
    /// Record a frame time
    pub fn record_frame_time(&mut self, duration: Duration) {
        self.frame_times.push(duration);
        
        // Keep only the last N samples
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }
    
    /// Get average system execution time
    pub fn get_average_system_time(&self, system_name: &str) -> Option<Duration> {
        let times = self.system_times.get(system_name)?;
        if times.is_empty() {
            return None;
        }
        
        let total: Duration = times.iter().sum();
        Some(total / times.len() as u32)
    }
    
    /// Get average frame time
    pub fn get_average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        Some(total / self.frame_times.len() as u32)
    }
    
    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        let avg_frame_time = self.get_average_frame_time();
        let fps = avg_frame_time.map(|t| 1.0 / t.as_secs_f64());
        
        let slowest_systems: Vec<_> = self.system_times
            .iter()
            .filter_map(|(name, times)| {
                if times.is_empty() { return None; }
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                Some((name.clone(), avg))
            })
            .collect();
        
        PerformanceSummary {
            average_fps: fps,
            average_frame_time: avg_frame_time,
            slowest_systems,
            total_systems: self.system_times.len(),
        }
    }
}

#[derive(Debug)]
pub struct PerformanceSummary {
    pub average_fps: Option<f64>,
    pub average_frame_time: Option<Duration>,
    pub slowest_systems: Vec<(String, Duration)>,
    pub total_systems: usize,
}

/// Macro for easy performance measurement
#[macro_export]
macro_rules! measure_performance {
    ($metrics:expr, $system_name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        $metrics.record_system_time($system_name.to_string(), duration);
        result
    }};
}

/// System to track frame times
pub fn track_frame_times(
    mut metrics: ResMut<PerformanceMetrics>,
    _time: Res<Time>,
) {
    let now = Instant::now();
    
    if let Some(last_start) = metrics.last_frame_start {
        let frame_time = now.duration_since(last_start);
        metrics.record_frame_time(frame_time);
    }
    
    metrics.last_frame_start = Some(now);
}

/// System to log performance metrics periodically
pub fn log_performance_metrics(
    metrics: Res<PerformanceMetrics>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    // Initialize timer on first run
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(5.0, TimerMode::Repeating));
    
    timer.tick(time.delta());
    
    if timer.just_finished() {
        let summary = metrics.get_summary();
        
        if let Some(fps) = summary.average_fps {
            info!("Performance: {:.1} FPS", fps);
        }
        
        if let Some(frame_time) = summary.average_frame_time {
            info!("Average frame time: {:.2}ms", frame_time.as_secs_f64() * 1000.0);
        }
        
        // Log slowest systems
        let mut systems = summary.slowest_systems;
        systems.sort_by(|a, b| b.1.cmp(&a.1));
        
        for (system_name, duration) in systems.iter().take(5) {
            info!("System '{}': {:.2}ms", system_name, duration.as_secs_f64() * 1000.0);
        }
    }
}

// Note: We can't implement Default for Timer due to orphan rules
// Instead, we'll create the timer directly in the system

/// Plugin for performance monitoring
pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerformanceMetrics>()
            .add_systems(Update, (
                track_frame_times,
                log_performance_metrics,
            ));
    }
} 