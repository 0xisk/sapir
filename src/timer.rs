use ark_std::end_timer;
use ark_std::perf_trace::inner::TimerInfo as ArkTimerInfo;
#[cfg(not(target_arch = "wasm32"))]
use ark_std::start_timer;
#[cfg(target_arch = "wasm32")]
use web_sys;

pub struct WebTimerInfo {
    label: &'static str,
}

pub enum TimerInfo {
    #[allow(dead_code)]
    Web(WebTimerInfo),
    #[allow(dead_code)]
    Cmd(ArkTimerInfo),
}

#[allow(dead_code)]
pub fn timer_start(label: &'static str) -> TimerInfo {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::time_with_label(label);
        TimerInfo::Web(WebTimerInfo { label })
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        TimerInfo::Cmd(start_timer!(|| label))
    }
}

#[allow(dead_code)]
pub fn timer_end(timer: TimerInfo) {
    match timer {
        TimerInfo::Web(t) => {
            web_sys::console::time_end_with_label(t.label);
        }
        TimerInfo::Cmd(t) => {
            end_timer!(t);
        }
    };
}

// Profiler is just a wrapper around timer.
// The only difference is that the time measured by a profiler
// is only rendered when the feature "profiler" is enabled .
// This is useful for embedding multiple timers in a function,
// but only rendering the time when necessary.
pub struct ProfilerInfo(Option<TimerInfo>);

pub fn profiler_start(_label: &'static str) -> ProfilerInfo {
    #[cfg(feature = "profiler")]
    {
        ProfilerInfo(Some(timer_start(_label)))
    }

    #[cfg(not(feature = "profiler"))]
    {
        ProfilerInfo(None)
    }
}

pub fn profiler_end(_profiler: ProfilerInfo) {
    #[cfg(feature = "profiler")]
    timer_end(_profiler.0.unwrap())
}