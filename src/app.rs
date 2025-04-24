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
    const INITIAL_SECONDS: u32 = 25 * 60;
    let remaining_seconds = create_signal(INITIAL_SECONDS);

    // 用於儲存計時器 Interval 的 handle，以便之後可以取消它
    // 使用 Rc<RefCell<>> 允許在回呼中修改 Option<Interval>
    let interval_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));

    // 閃爍效果的狀態和計時器
    let is_blinking = create_signal(false);
    let blink_visible = create_signal(true);
    let blink_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));

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

    // 啟動閃爍提示
    let trigger_blink = move |blink_handle: Rc<RefCell<Option<Interval>>>,
                              is_blinking: Signal<bool>,
                              blink_visible: Signal<bool>| {
        if !is_blinking.get() {
            is_blinking.set(true);
            blink_visible.set(true);
            let blink_vis = blink_visible.clone();
            let timer = Interval::new(500, move || {
                blink_vis.set(!blink_vis.get());
            });
            *blink_handle.borrow_mut() = Some(timer);
        }
    };

    let start_timer = move |interval_handle: Rc<RefCell<Option<Interval>>>,
                            remaining_seconds: Signal<u32>,
                            blink_handle: Rc<RefCell<Option<Interval>>>,
                            is_blinking: Signal<bool>,
                            blink_visible: Signal<bool>| {
        // 先取消舊的計時器
        if let Some(handle) = interval_handle.borrow_mut().take() {
            drop(handle);
        }

        let interval_handle_clone = interval_handle.clone();
        let remaining_seconds_clone = remaining_seconds.clone();
        let blink_handle_clone = blink_handle.clone();
        let is_blinking_clone = is_blinking.clone();
        let blink_visible_clone = blink_visible.clone();

        // 每秒遞減
        let new_timer = Interval::new(1000, move || {
            let secs = remaining_seconds_clone.get();
            if secs > 0 {
                remaining_seconds_clone.set(secs - 1);
            } else {
                // 取消主計時器
                if let Some(handle) = interval_handle_clone.borrow_mut().take() {
                    drop(handle);
                }

                // 呼叫閃爍提示
                trigger_blink(
                    blink_handle_clone.clone(),
                    is_blinking_clone.clone(),
                    blink_visible_clone.clone(),
                );
            }
        });

        *interval_handle.borrow_mut() = Some(new_timer);
    };

    // --- 設定計時器邏輯 ---
    // 使用 on_mount 在元件掛載時啟動計時器
    let interval_handle_on_mount = interval_handle.clone();
    let remaining_seconds_on_mount = remaining_seconds.clone();
    let blink_handle_on_mount = blink_handle.clone();
    let is_blinking_on_mount = is_blinking.clone();
    let blink_visible_on_mount = blink_visible.clone();
    on_mount(move || {
        start_timer(
            interval_handle_on_mount,
            remaining_seconds_on_mount,
            blink_handle_on_mount.clone(),
            is_blinking_on_mount.clone(),
            blink_visible_on_mount.clone(),
        );
    });

    // --- 清理計時器 ---
    // 使用 on_cleanup 確保在元件卸載時取消計時器，防止內存洩漏
    let interval_handle_on_cleanup = interval_handle.clone();
    let blink_handle_on_cleanup = blink_handle.clone();
    on_cleanup(move || {
        if let Some(handle) = interval_handle_on_cleanup.take() {
            drop(handle); // 取消 Interval
        }
        if let Some(handle) = blink_handle_on_cleanup.borrow_mut().take() {
            drop(handle); // 取消閃爍 Interval
        }
    });

    // --- UI 視圖 ---
    let interval_handle_clone = interval_handle.clone();
    let remaining_seconds_clone = remaining_seconds.clone();
    let is_blinking_clone = is_blinking.clone();
    let blink_visible_clone = blink_visible.clone();
    let blink_handle_clone = blink_handle.clone();

    view! {
        div(class="timer-container") {
            p(
                class="timer-display",
                style=move || {
                    // 添加滑鼠指針和防止文字被選取
                    let base_style = "cursor: pointer; user-select: none;";

                    // 添加顏色樣式 - 當閃爍時（計時結束）顯示紅色文字
                    let color_style = if is_blinking_clone.get() {
                        "color: red;"
                    } else {
                        ""  // 預設顏色
                    };

                    // 結合可見性樣式
                    if is_blinking_clone.get() && !blink_visible_clone.get() {
                        format!("{} {} visibility: hidden", base_style, color_style)
                    } else {
                        format!("{} {} visibility: visible", base_style, color_style)
                    }
                },
                on:click=move |_| {
                    // 重置時取消閃爍效果
                    if is_blinking_clone.get() {
                        is_blinking_clone.set(false);
                        blink_visible_clone.set(true);
                        if let Some(handle) = blink_handle_clone.borrow_mut().take() {
                            drop(handle);
                        }
                    }

                    remaining_seconds_clone.set(INITIAL_SECONDS);
                    start_timer(
                        interval_handle_clone.clone(),
                        remaining_seconds_clone.clone(),
                        blink_handle_clone.clone(),
                        is_blinking_clone.clone(),
                        blink_visible_clone.clone(),
                    );
                }
            ) {
                (formatted_time)
            }
        }
    }
}
