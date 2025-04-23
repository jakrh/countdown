use gloo_timers::callback::Interval;
use std::cell::RefCell;
use std::rc::Rc;
use sycamore::prelude::*;
// 如果你想在瀏覽器主控台印出訊息
// use web_sys::console;

#[component]
pub fn App() -> View {
    // --- 倒數計時器狀態 ---
    // 設定初始倒數時間 (例如：5 分鐘 = 300 秒)
    const INITIAL_SECONDS: u32 = 8;
    let remaining_seconds = create_signal(INITIAL_SECONDS);

    // 用於儲存計時器 Interval 的 handle，以便之後可以取消它
    // 使用 Rc<RefCell<>> 允許在回呼中修改 Option<Interval>
    let interval_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));

    // --- 計算顯示的格式 (MM:SS) ---
    // 使用 create_memo 以便只有在 remaining_seconds 變化時才重新計算
    let formatted_time = create_memo(move || {
        let total_secs = remaining_seconds.get();
        if total_secs == 0 {
            return "00:00".to_string(); // 時間到
        }
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    });

    let start_timer = |interval_handle: Rc<RefCell<Option<Interval>>>,
                       remaining_seconds: Signal<u32>| {
        if let Some(handle) = interval_handle.borrow_mut().take() {
            drop(handle);
        }
        let interval_handle_clone = interval_handle.clone();
        let remaining_seconds_clone = remaining_seconds;
        let new_timer = Interval::new(1000, move || {
            let current_secs = remaining_seconds_clone.get();
            if current_secs > 0 {
                remaining_seconds_clone.set(current_secs - 1);
            } else {
                if let Some(handle) = interval_handle_clone.borrow_mut().take() {
                    drop(handle);
                }
            }
        });
        *interval_handle.borrow_mut() = Some(new_timer);
    };

    // --- 設定計時器邏輯 ---
    // 使用 on_mount 在元件掛載時啟動計時器
    let interval_handle_on_mount = interval_handle.clone();
    let remaining_seconds_on_mount = remaining_seconds.clone();
    on_mount(move || {
        // start_timer();
        start_timer(interval_handle_on_mount, remaining_seconds_on_mount);
    });

    // --- 清理計時器 ---
    // 使用 on_cleanup 確保在元件卸載時取消計時器，防止內存洩漏
    let interval_handle_on_cleanup = interval_handle.clone();
    on_cleanup(move || {
        if let Some(handle) = interval_handle_on_cleanup.take() {
            drop(handle); // 取消 Interval
                          // console::log_1(&"Timer cleaned up.".into());
        }
    });

    // --- UI 視圖 ---
    let interval_handle_clone = interval_handle.clone();
    let remaining_seconds_clone = remaining_seconds.clone(); // Signal 是 Copy 的
    view! {
        div(class="timer-container") {
            p(class="timer-display") {
                (formatted_time)
            }
            button(on:click=move |_| {
                remaining_seconds_clone.set(INITIAL_SECONDS);
                start_timer(interval_handle_clone.clone(), remaining_seconds_clone);
            }) { "Reset" }
        }
    }
}
